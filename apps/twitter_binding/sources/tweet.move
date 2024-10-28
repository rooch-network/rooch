//This module is deprecated, please use tweet_v2.
module twitter_binding::tweet {

    use std::option::{Option};
    use std::string::{String};

    use moveos_std::json;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::signer;
    
    const ErrorTweetNotFound: u64 = 1;
    const ErrorTweetOwnerNotMatch: u64 = 2;
    const ErrorInvalidTweetJson: u64 = 3;
    const ErrorDeprecated : u64 = 4;

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
    struct TweetData has store, copy, drop{
        id: String,
        text: String,
        note_tweet: Option<NoteTweet>,
        author_id: String,
        created_at: String,
    }
    
    /// The tweet object
    /// No `store` ability, so the user can not transfer the tweet object to other address
    struct Tweet has key{
        id: String,
        text: String,
        note_tweet: Option<NoteTweet>,
        author_id: String,
        created_at: String,
    }

    // =========================== Tweet object functions ===========================

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
        let Tweet{id:_, text:_, note_tweet:_, author_id:_, created_at:_} = tweet;
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

}