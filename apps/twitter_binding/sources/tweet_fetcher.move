module twitter_binding::tweet_fetcher {

    use std::option;
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::json;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::event;
    
    use twitter_binding::twitter_account;
    use twitter_binding::tweet::{Self, Tweet};

    use verity::oracles;
    
    const TWEET_URL: vector<u8> = b"https://api.twitter.com/2/tweets/";
    const TWEET_URL_PARAM: vector<u8> = b"?tweet.fields=id,created_at,author_id,text,entities,attachments,note_tweet";
    const TWEET_METHOD: vector<u8> = b"GET";
    const TWEET_HEADERS: vector<u8> = b"{}";
    const TWEET_BODY: vector<u8> = b"{}";
    /// The jq query to parse the tweet, `.` means the root object
    const PICK: vector<u8> = b".data";

    const ORACLE_ADDRESS: address = @0x694cbe655b126e9e6a997e86aaab39e538abf30a8c78669ce23a98740b47b65d;

    const ErrorInvalidRequestID: u64 = 1;
    const ErrorInvalidResponse: u64 = 2;
    const ErrorTooManyPendingRequests: u64 = 3;

    struct FetchQueue has key {
        request_queue: vector<ObjectID>,
    }  

    struct TweetProcessEvent has store, copy, drop{
        tweet_id: String,
        request_id: ObjectID,
        status: u8,
    }

    const TWEET_STATUS_FETCHING: u8 = 0;
    public fun tweet_status_fetching(): u8 {
        TWEET_STATUS_FETCHING
    }
    const TWEET_STATUS_PROCESSED: u8 = 1;
    public fun tweet_status_processed(): u8 {
        TWEET_STATUS_PROCESSED
    }
    const TWEET_STATUS_FETCH_FAILED: u8 = 2;
    public fun tweet_status_fetch_failed(): u8 {
        TWEET_STATUS_FETCH_FAILED
    }
    const TWEET_STATUS_PROCESS_FAILED: u8 = 3;
    public fun tweet_status_process_failed(): u8 {
        TWEET_STATUS_PROCESS_FAILED
    }

    struct FetchResult has store, copy, drop{
        /// The tweet id
        tweet_id: String,
        /// The tweet object id
        tweet_object_id: ObjectID,
        /// The request id
        request_id: ObjectID,
        /// The tweet status
        status: u8,
    }

    fun init(){
        let fetch_queue = FetchQueue{
            request_queue: vector::empty(),
        };
        let fetch_queue_obj = object::new_named_object(fetch_queue);
        object::transfer_extend(fetch_queue_obj, @twitter_binding);
    }


    public entry fun fetch_tweet_entry(id: String){
        let _fetch_result = fetch_tweet(id);
    }

    public fun fetch_tweet(tweet_id: String): FetchResult {
        let tweet_object_id = tweet::tweet_object_id(tweet_id);
        let fetch_queue_obj = borrow_mut_fetch_queue_obj();
        assert!(vector::length(&object::borrow(fetch_queue_obj).request_queue) < 100, ErrorTooManyPendingRequests);
        if (object::contains_field(fetch_queue_obj, tweet_id)){
            let request_id: ObjectID = *object::borrow_field(fetch_queue_obj, tweet_id);
            let status = try_process_request(request_id);
            return FetchResult {
                tweet_id,
                tweet_object_id,
                request_id,
                status,
            }
        };

        let url = string::utf8(TWEET_URL);
        string::append(&mut url, tweet_id);
        string::append(&mut url, string::utf8(TWEET_URL_PARAM));
        let method = string::utf8(TWEET_METHOD);
        let headers = string::utf8(TWEET_HEADERS);
        let body = string::utf8(TWEET_BODY);
        //The jq query to parse the tweet
        let pick = string::utf8(PICK);
        let http_request = oracles::build_request(url, method, headers, body);
        let request_id = oracles::new_request(http_request, pick, ORACLE_ADDRESS, oracles::with_notify(@twitter_binding, b"tweet_fetcher::check_request_queue"));
        let fetch_result = FetchResult{
            tweet_id,
            tweet_object_id,
            request_id,
            status: TWEET_STATUS_FETCHING,
        };
        // Record the tweet id => request id
        object::add_field(fetch_queue_obj, tweet_id, request_id);
        // Record the request id => tweet id
        object::add_field(fetch_queue_obj, request_id, tweet_id);
        vector::push_back(&mut object::borrow_mut(fetch_queue_obj).request_queue, request_id);
        fetch_result
    }

    /// The oracle callback function
    public entry fun check_request_queue() {
        let fetch_queue_obj = borrow_mut_fetch_queue_obj();
        let request_queue = *&object::borrow(fetch_queue_obj).request_queue;
        vector::for_each(request_queue, |request_id|{
            process_request_internal(request_id, fetch_queue_obj);
        });
    }

    public fun try_process_request(request_id: ObjectID): u8{
        let fetch_queue_obj = borrow_mut_fetch_queue_obj();
        process_request_internal(request_id, fetch_queue_obj)
    }

    fun process_request_internal(request_id: ObjectID, fetch_queue_obj: &mut Object<FetchQueue>): u8{
        assert!(object::contains_field(fetch_queue_obj, request_id), ErrorInvalidRequestID);
        let tweet_id: String = *object::borrow_field(fetch_queue_obj, request_id);
        let tweet_object_id = tweet::tweet_object_id(tweet_id);
        // The tweet object already exists, the request should be finished and successful
        if (object::exists_object_with_type<Tweet>(tweet_object_id)){
            return TWEET_STATUS_PROCESSED
        };

        let response_http_status = oracles::get_response_status(&request_id);
        // The request is not finished
        if (response_http_status == 0){
            return TWEET_STATUS_FETCHING
        };

        let status = if (response_http_status == 200){
            let response_opt = oracles::get_response(&request_id);
            let response = option::destroy_some(response_opt);
            // The response is a JSON string including the tweet json data
            let json_str = json::from_json<String>(string::into_bytes(response));
            let tweet_data_opt = tweet::parse_tweet_data(string::into_bytes(json_str));
            if (option::is_some(&tweet_data_opt)){
                let tweet_data = option::destroy_some(tweet_data_opt);
                let author_id = *tweet::tweet_data_author_id(&tweet_data);
                let tweet_obj = tweet::new_tweet_object(tweet_data);
                let author_address_opt = twitter_account::resolve_address_by_author_id(author_id);
                // If the author address is not found, the tweet owner is the twitter binding address
                // The author can claim the tweet by himself after binding his twitter account
                let owner_address = option::destroy_with_default(author_address_opt, @twitter_binding);
                tweet::transfer_tweet_object_internal(tweet_obj, owner_address);
                TWEET_STATUS_PROCESSED
            }else{
                TWEET_STATUS_PROCESS_FAILED
            }
        } else {
            TWEET_STATUS_FETCH_FAILED
        };
        let event = TweetProcessEvent{
            tweet_id,
            request_id,
            status,
        };
        event::emit(event);
        // Clear the mapping between tweet id and request id
        let _request_id: ObjectID = object::remove_field(fetch_queue_obj, tweet_id);
        let _tweet_id: String = object::remove_field(fetch_queue_obj, request_id);
        // Remove the request id from the request queue
        let fetch_queue = object::borrow_mut(fetch_queue_obj);
        vector::remove_value(&mut fetch_queue.request_queue, &request_id);
        status
    }

    public fun unpack_fetch_result(fetch_result: FetchResult): (String, ObjectID, ObjectID, u8){
        let FetchResult {
            tweet_id,
            tweet_object_id,
            request_id,
            status,
        } = fetch_result;
        (tweet_id, tweet_object_id, request_id, status)
    }

    // =========================== Internal functions ===========================

    fun borrow_fetch_queue_obj(): &Object<FetchQueue>{
        let fetch_queue_obj_id = object::named_object_id<FetchQueue>();
        object::borrow_object<FetchQueue>(fetch_queue_obj_id)
    }

    fun borrow_mut_fetch_queue_obj(): &mut Object<FetchQueue>{
        let fetch_queue_obj_id = object::named_object_id<FetchQueue>();
        object::borrow_mut_object_extend<FetchQueue>(fetch_queue_obj_id)
    }

}