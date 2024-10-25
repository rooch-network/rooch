module twitter_binding::tweet_v2 {

    use std::option::{Self, Option};
    use std::string::{String};
    use std::vector;

    use moveos_std::json;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::signer;
    
    const ErrorTweetNotFound: u64 = 1;
    const ErrorTweetOwnerNotMatch: u64 = 2;
    const ErrorInvalidTweetJson: u64 = 3;

    friend twitter_binding::twitter_account;
    friend twitter_binding::tweet_fetcher;

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
    struct Referenced_tweet has store, copy, drop{
        id: String,
        type: String,
    }

    #[data_struct]
    struct TweetData has store, copy, drop{
        id: String,
        text: String,
        note_tweet: Option<NoteTweet>,
        author_id: String,
        created_at: String,
        entities: Option<Entities>,
        referenced_tweets: vector<Referenced_tweet>,
        edit_history_tweet_ids: vector<String>,
    }
    
    /// The tweet object
    /// No `store` ability, so the user can not transfer the tweet object to other address
    struct Tweet has key{
        id: String,
        text: String,
        entities: Entities,
        author_id: String,
        created_at: String,
        referenced_tweets: vector<Referenced_tweet>,
        edit_history_tweet_ids: vector<String>,
    }

    // =========================== Tweet object functions ===========================

    public(friend) fun new_tweet_object(tweet_data: TweetData): Object<Tweet>{
        let TweetData{id, text, note_tweet, author_id, created_at, entities: entities_opt, referenced_tweets, edit_history_tweet_ids} = tweet_data;
        let NoteTweet{text: note_text, entities: note_entities} = if(option::is_some(&note_tweet)) {
            option::destroy_some(note_tweet)
        } else {
            NoteTweet{text, entities: default_entities()}
        };
        //The text in NoteTweet include the text
        //We merge the entities in NoteTweet and entities in TweetData
        let entities = if(option::is_some(&entities_opt)) {
            let entities = option::destroy_some(entities_opt);
            merge_entities(&mut entities, note_entities);
            entities
        } else {
            note_entities
        };
        let tweet = Tweet{id, text: note_text, entities, author_id, created_at, referenced_tweets, edit_history_tweet_ids};
        object::new_with_id(id, tweet)
    }

    /// Get the object id of the tweet object
    public fun tweet_object_id(id: String): ObjectID {
        object::custom_object_id<String, Tweet>(id)
    }

    public fun borrow_tweet_object(id: String): &Object<Tweet>{
        let tweet_obj_id = tweet_object_id(id);
        assert!(object::exists_object_with_type<Tweet>(tweet_obj_id), ErrorTweetNotFound);
        object::borrow_object<Tweet>(tweet_obj_id)
    }

    public fun exists_tweet_object(id: String): bool {
        object::exists_object_with_type<Tweet>(tweet_object_id(id))
    }

    public(friend) fun take_tweet_object_internal(tweet_obj_id: ObjectID): Object<Tweet>{
        object::take_object_extend<Tweet>(tweet_obj_id)
    }

    public(friend) fun transfer_tweet_object_internal(tweet_obj: Object<Tweet>, owner: address){
        object::transfer_extend(tweet_obj, owner);
    }

    public(friend) fun remove_tweet_object_internal(tweet_obj_id: ObjectID){
        let tweet_obj = object::take_object_extend<Tweet>(tweet_obj_id);
        let tweet = object::remove(tweet_obj);
        drop_tweet(tweet);
    }

    /// Remove the tweet object, the tweet owner can remove the tweet object
    public entry fun remove_tweet_object(owner: &signer, tweet_obj_id: ObjectID){
        let owner_address = signer::address_of(owner);
        let tweet_obj = object::take_object_extend<Tweet>(tweet_obj_id);
        assert!(object::owner(&tweet_obj) == owner_address, ErrorTweetOwnerNotMatch);
        let tweet = object::remove(tweet_obj);
        drop_tweet(tweet);
    }

    fun drop_tweet(tweet: Tweet){
        let Tweet{id:_, text:_, entities:_, author_id:_, created_at:_, referenced_tweets:_, edit_history_tweet_ids:_} = tweet;
    }

    // ========================== TweetData functions ==========================

    public fun parse_tweet_data(tweet_data_json: vector<u8>): Option<TweetData> { 
        json::from_json_option<TweetData>(tweet_data_json)
    }
    
    public fun tweet_data_id(tweet_data: &TweetData): &String {
        &tweet_data.id
    }

    public fun tweet_data_text(tweet_data: &TweetData): &String {
        &tweet_data.text
    }

    public fun tweet_data_note_tweet(tweet_data: &TweetData): &Option<NoteTweet> {
        &tweet_data.note_tweet
    }

    public fun tweet_data_author_id(tweet_data: &TweetData): &String {
        &tweet_data.author_id
    }

    public fun tweet_data_created_at(tweet_data: &TweetData): &String {
        &tweet_data.created_at
    }

    public fun tweet_data_entities(tweet_data: &TweetData): &Option<Entities> {
        &tweet_data.entities
    }

    public fun tweet_data_referenced_tweets(tweet_data: &TweetData): &vector<Referenced_tweet> {
        &tweet_data.referenced_tweets
    }

    public fun tweet_data_edit_history_tweet_ids(tweet_data: &TweetData): &vector<String> {
        &tweet_data.edit_history_tweet_ids
    }

    // ================================ Tweet functions ================================

    public fun tweet_id(tweet: &Tweet): &String {
        &tweet.id
    }

    public fun tweet_text(tweet: &Tweet): &String {
        &tweet.text
    }

    public fun tweet_entities(tweet: &Tweet): &Entities {
        &tweet.entities
    }

    public fun tweet_author_id(tweet: &Tweet): &String {
        &tweet.author_id
    }   

    public fun tweet_created_at(tweet: &Tweet): &String {
        &tweet.created_at
    }

    public fun tweet_referenced_tweets(tweet: &Tweet): &vector<Referenced_tweet> {
        &tweet.referenced_tweets
    }

    public fun tweet_edit_history_tweet_ids(tweet: &Tweet): &vector<String> {
        &tweet.edit_history_tweet_ids
    }

    // ========================== NoteTweet functions ==========================

    public fun tweet_note_tweet_text(note_tweet: &NoteTweet): &String {
        &note_tweet.text
    }

    public fun tweet_note_tweet_entities(note_tweet: &NoteTweet): &Entities {
        &note_tweet.entities
    }

    // ========================== Entities functions ==========================


    public fun default_entities(): Entities {
        Entities{urls: vector::empty(), mentions: vector::empty(), hashtags: vector::empty(), cashtags: vector::empty()}
    }

    public fun merge_entities(entities: &mut Entities, other: Entities){
        let Entities{urls, mentions, hashtags, cashtags} = other;
        vector::append(&mut entities.urls, urls);
        vector::append(&mut entities.mentions, mentions);
        vector::append(&mut entities.hashtags, hashtags);
        vector::append(&mut entities.cashtags, cashtags);
    }

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

    // =========================== Test functions ===========================

    #[test_only]
    public fun new_tweet_object_for_test(json_str: vector<u8>): Object<Tweet>{
        let tweet_data_option = parse_tweet_data(json_str);
        assert!(option::is_some(&tweet_data_option), ErrorInvalidTweetJson);
        let tweet_data = option::destroy_some(tweet_data_option);
        new_tweet_object(tweet_data)
    }

    #[test]
    fun test_parse_json(){
        // Generate a simplified fake tweet data for testing JSON parsing
        let test_tweet_json: vector<u8> = b"{\"note_tweet\": {\"text\": \"This is a test tweet\",\"entities\": {\"mentions\": [{\"start\": 0,\"end\": 9,\"username\": \"testuser\",\"id\": \"123456789\"}],\"urls\": [{\"start\": 10,\"end\": 33,\"url\": \"https://t.co/abcdefg\",\"expanded_url\": \"https://example.com\",\"display_url\": \"example.com\"}]}},\"author_id\": \"987654321\",\"id\": \"1234567890123456789\",\"text\": \"This is a test tweet https://t.co/abcdefg\",\"created_at\": \"2024-01-01T00:00:00.000Z\",\"edit_history_tweet_ids\": [\"1234567890123456789\"]}";
        let tweet_option = json::from_json_option<TweetData>(test_tweet_json);
        assert!(option::is_some(&tweet_option), 1);
        let _tweet = option::destroy_some(tweet_option);
        //std::debug::print(&tweet);
    }

    #[test]
    fun test_parse_json2(){
        let test_tweet_json: vector<u8> = b"{\"id\": \"1844391830802341950\",\"created_at\": \"2024-10-10T14:57:34.000Z\",\"edit_history_tweet_ids\": [\"1844391830802341950\"],\"author_id\": \"1045398019351425026\",\"text\": \"test\"}";
        let tweet_option = json::from_json_option<TweetData>(test_tweet_json);
        assert!(option::is_some(&tweet_option), 1);
        let _tweet = option::destroy_some(tweet_option);
        //std::debug::print(&tweet);
    }

    #[test]
    fun test_parse_json3(){
        let test_tweet_json: vector<u8> = b"{\"id\": \"1844391830802341950\",\"created_at\": \"2024-10-10T14:57:34.000Z\",\"edit_history_tweet_ids\": [\"1844391830802341950\"],\"author_id\": \"1045398019351425026\",\"text\": \"test #RoochNetwork\", \"entities\": {\"hashtags\": [{\"start\": 5,\"end\": 18,\"tag\": \"RoochNetwork\"}]},\"referenced_tweets\": [{\"id\": \"1844391830802341950\",\"type\": \"retweeted\"}]}";
        let tweet_option = json::from_json_option<TweetData>(test_tweet_json);
        assert!(option::is_some(&tweet_option), 1);
        let _tweet = option::destroy_some(tweet_option);
        //std::debug::print(&tweet);
    }

}