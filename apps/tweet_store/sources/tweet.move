module tweet_store::tweet {

    use std::option::{Self, Option};
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::json;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::event;
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

    #[data_struct]
    struct Tag has store, copy, drop{
        start: u64,
        end: u64,
        tag: String,
    }

    #[data_struct]
    struct Mention has store, copy, drop{
        start: u64,
        end: u64,
        username: String,
        id: String,
    }

    #[data_struct]
    struct Url has store, copy, drop{
        start: u64,
        end: u64,
        url: String,
        expanded_url: String,
        display_url: String,
    }

    #[data_struct]
    struct Entities has store, copy, drop{
        urls: vector<Url>,
        mentions: vector<Mention>,
        hashtags: vector<Tag>,
        cashtags: vector<Tag>,
    }

    #[data_struct]
    struct NoteTweet has store, copy, drop{
        text: String,
        entities: Entities,
    }
    
    #[data_struct]
    struct Tweet has key, copy, drop{
        id: String,
        text: String,
        note_tweet: Option<NoteTweet>,
        author_id: String,
        created_at: String,
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
        object::transfer_extend(fetch_queue_obj, @tweet_store);
    }

    fun borrow_fetch_queue_obj(): &Object<FetchQueue>{
        let fetch_queue_obj_id = object::named_object_id<FetchQueue>();
        object::borrow_object<FetchQueue>(fetch_queue_obj_id)
    }

    fun borrow_mut_fetch_queue_obj(): &mut Object<FetchQueue>{
        let fetch_queue_obj_id = object::named_object_id<FetchQueue>();
        object::borrow_mut_object_extend<FetchQueue>(fetch_queue_obj_id)
    }

    public fun tweet_object_id(id: String): ObjectID {
        object::custom_object_id<String, Tweet>(id)
    }

    public fun exists_tweet_object(id: String): bool {
        object::exists_object_with_type<Tweet>(tweet_object_id(id))
    }

    public entry fun fetch_tweet_entry(id: String){
        let _fetch_result = fetch_tweet(id);
    }

    public fun fetch_tweet(tweet_id: String): FetchResult {
        let tweet_object_id = tweet_object_id(tweet_id);
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
        let request_id = oracles::new_request(http_request, pick, ORACLE_ADDRESS, oracles::with_notify(@tweet_store, b"tweet::check_request_queue"));
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
        let tweet_object_id = tweet_object_id(tweet_id);
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
            let tweet_opt = json::from_json_option<Tweet>(string::into_bytes(json_str));
            if (option::is_some(&tweet_opt)){
                let tweet = option::destroy_some(tweet_opt);
                let tweet_obj = object::new_with_id(tweet_id, tweet);
                object::transfer_extend(tweet_obj, @tweet_store);
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

    // ================================ Tweet functions ================================

    public fun tweet_text(tweet: &Tweet): &String {
        &tweet.text
    }

    public fun tweet_author_id(tweet: &Tweet): &String {
        &tweet.author_id
    }   

    public fun tweet_created_at(tweet: &Tweet): &String {
        &tweet.created_at
    }

    public fun tweet_note_tweet(tweet: &Tweet): &Option<NoteTweet> {
        &tweet.note_tweet
    }

    // ========================== NoteTweet functions ==========================

    public fun tweet_note_tweet_text(note_tweet: &NoteTweet): &String {
        &note_tweet.text
    }

    public fun tweet_note_tweet_entities(note_tweet: &NoteTweet): &Entities {
        &note_tweet.entities
    }

    // ========================== Entities functions ==========================

    public fun tweet_entities_urls(entities: &Entities): &vector<Url> {
        &entities.urls
    }

    public fun tweet_entities_mentions(entities: &Entities): &vector<Mention> {
        &entities.mentions
    }

    public fun tweet_entities_hashtags(entities: &Entities): &vector<Tag> {
        &entities.hashtags
    }

    public fun tweet_entities_cashtags(entities: &Entities): &vector<Tag> {
        &entities.cashtags
    }

    // ========================== Url functions ==========================

    public fun tweet_url_start(url: &Url): u64 {
        url.start
    }

    public fun tweet_url_end(url: &Url): u64 {
        url.end
    }

    public fun tweet_url_url(url: &Url): &String {
        &url.url
    }

    public fun tweet_url_expanded_url(url: &Url): &String {
        &url.expanded_url
    }

    public fun tweet_url_display_url(url: &Url): &String {
        &url.display_url
    }

    // ========================== Mention functions ==========================

    public fun tweet_mention_start(mention: &Mention): u64 {
        mention.start
    }   

    public fun tweet_mention_end(mention: &Mention): u64 {
        mention.end
    }

    public fun tweet_mention_username(mention: &Mention): &String {
        &mention.username
    }

    public fun tweet_mention_id(mention: &Mention): &String {
        &mention.id
    }

    // ========================== Tag functions ==========================

    public fun tweet_tag_start(tag: &Tag): u64 {
        tag.start
    }

    public fun tweet_tag_end(tag: &Tag): u64 {
        tag.end
    }

    public fun tweet_tag_tag(tag: &Tag): &String {
        &tag.tag
    }


    #[test]
    fun test_parse_json(){
        // Generate a simplified fake tweet data for testing JSON parsing
        let test_tweet_json: vector<u8> = b"{\"note_tweet\": {\"text\": \"This is a test tweet\",\"entities\": {\"mentions\": [{\"start\": 0,\"end\": 9,\"username\": \"testuser\",\"id\": \"123456789\"}],\"urls\": [{\"start\": 10,\"end\": 33,\"url\": \"https://t.co/abcdefg\",\"expanded_url\": \"https://example.com\",\"display_url\": \"example.com\"}]}},\"author_id\": \"987654321\",\"id\": \"1234567890123456789\",\"text\": \"This is a test tweet https://t.co/abcdefg\",\"created_at\": \"2024-01-01T00:00:00.000Z\",\"edit_history_tweet_ids\": [\"1234567890123456789\"]}";
        let tweet_option = json::from_json_option<Tweet>(test_tweet_json);
        assert!(option::is_some(&tweet_option), 1);
        let _tweet = option::destroy_some(tweet_option);
        //std::debug::print(&tweet);
    }

    #[test]
    fun test_parse_json2(){
        let test_tweet_json: vector<u8> = b"{\"id\": \"1844391830802341950\",\"created_at\": \"2024-10-10T14:57:34.000Z\",\"edit_history_tweet_ids\": [\"1844391830802341950\"],\"author_id\": \"1045398019351425026\",\"text\": \"how software actually works for 99% of engineers: someone way smarter than you solved a really hard problem and now you build on top of their solution like adult legos and think you're a genius\"}";
        let tweet_option = json::from_json_option<Tweet>(test_tweet_json);
        assert!(option::is_some(&tweet_option), 1);
        let _tweet = option::destroy_some(tweet_option);
        //std::debug::print(&tweet);
    }
}