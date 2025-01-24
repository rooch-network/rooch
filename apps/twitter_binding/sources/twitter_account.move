module twitter_binding::twitter_account {

    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option};

    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::table::{Self, Table};
    use moveos_std::event;
    use moveos_std::tx_context::{sender};
    use moveos_std::signer;

    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;
    use rooch_framework::gas_coin::{RGas};

    use twitter_binding::tweet_v2::{Self, Tweet};
    use app_admin::admin::{AdminCap};

    const TWITTER_ACCOUNT_BINDING_MESSAGE_PREFIX: vector<u8> = b"BTC:";
    const TWITTER_ACCOUNT_BINDING_HASH_TAG: vector<u8> = b"RoochNetwork";
    const ADDRESS_SPLIT_CHARS: vector<u8> = b" #,.\n\r";

    const BITCOIN_TAPROOT_ADDRESS_PREFIX_MAINNET: vector<u8> = b"bc1";

    const ErrorTweetNotFound: u64 = 1;
    const ErrorAccountAlreadyBound: u64 = 2;
    const ErrorAuthorAddressNotFound: u64 = 3;
    const ErrorTweetBindingMessageInvalidPrefix: u64 = 4;
    const ErrorTweetBindingMessageMissingHashtag: u64 = 5;
    const ErrorTweetBindingMessageInvalidAddress: u64 = 6;

    const INIT_GAS_AMOUNT: u256 = 1000000_00000000;

    const REWARD_RGAS_AMOUNT: u256 = 50_00000000;

    //Deprecated
    struct TwitterBindingErrorEvent has store, drop, copy {
        tweet_id: String,
        author_id: String,
        error: String,
    }

    struct TwitterBindingEvent has store, drop, copy {
        tweet_obj_id: ObjectID,
        author_id: String,
        bitcoin_address: BitcoinAddress,
        account_address: address,
    }

    /// The twitter account object
    struct TwitterAccount has key {
        /// The x.com account id
        id: String,
        binding_tweet_obj_id: ObjectID,
    }

    struct TwitterAccountMapping has key {
        /// The mapping between the x.com account id and the account address
        account_to_address: Table<String, address>,
        /// The mapping between the account address and the account id
        address_to_account: Table<address, String>,
    }

    struct TwitterRGasFaucet has key {
        /// First binding reward gas store
        rgas_store: Object<CoinStore<RGas>>,
        /// First binding reward gas claim records
        /// Twitter user id -> gas amount
        claim_records: Table<String, u256>,
        /// Is the faucet open
        is_open: bool,
    }

    fun init(){
        let module_signer = signer::module_signer<TwitterAccountMapping>();
        let rgas_store = coin_store::create_coin_store<RGas>();
        let rgas_balance = account_coin_store::balance<RGas>(@twitter_binding);
        let faucet_gas_amount = if(rgas_balance > INIT_GAS_AMOUNT) {
            INIT_GAS_AMOUNT
        } else {
            rgas_balance/3
        };
        Self::deposit_to_rgas_store(&module_signer, &mut rgas_store, faucet_gas_amount);
        let twitter_rgas_faucet = TwitterRGasFaucet{
            rgas_store,
            claim_records: table::new(),
            is_open: true,
        };
        let twitter_rgas_faucet_obj = object::new_named_object(twitter_rgas_faucet);
        object::transfer_extend(twitter_rgas_faucet_obj, @twitter_binding);
        let twitter_account_mapping = TwitterAccountMapping{
            account_to_address: table::new(),
            address_to_account: table::new(),
        };
        let mapping_obj = object::new_named_object(twitter_account_mapping);
        object::transfer_extend(mapping_obj, @twitter_binding);
    }

    /// Resolve address by author id
    public fun resolve_address_by_author_id(author_id: String): Option<address> {
        let mapping = borrow_twitter_account_mapping();
        if (table::contains(&mapping.account_to_address, author_id)){
            return option::some(*table::borrow(&mapping.account_to_address, author_id))
        };
        option::none()
    }

    /// Resolve address by author id batch, if the address not found, the address will be 0x0
    public fun resolve_address_by_author_id_batch(author_ids: vector<String>): vector<address> {
        let mapping = borrow_twitter_account_mapping();
        let len = vector::length(&author_ids);
        let results = vector::empty<address>();
        let i = 0;
        while (i < len){
            let author_id = *vector::borrow(&author_ids, i);
            let addr = if (table::contains(&mapping.account_to_address, author_id)){
                *table::borrow(&mapping.account_to_address, author_id)
            }else{
                @0x0
            };
            vector::push_back(&mut results, addr);
            i = i + 1;
        };
        results
    }

    /// Resolve author id by address
    public fun resolve_author_id_by_address(address: address): Option<String> {
        let mapping = borrow_twitter_account_mapping();
        if (table::contains(&mapping.address_to_account, address)){
            return option::some(*table::borrow(&mapping.address_to_account, address))
        };
        option::none()
    }

    /// Resolve author id by address batch, if the address not found, the author id will be empty string
    public fun resolve_author_id_by_address_batch(addresses: vector<address>): vector<String> {
        let mapping = borrow_twitter_account_mapping();
        let len = vector::length(&addresses);
        let results = vector::empty<String>();
        let i = 0;
        while (i < len){
            let addr = *vector::borrow(&addresses, i);
            let author_id = if (table::contains(&mapping.address_to_account, addr)){
                *table::borrow(&mapping.address_to_account, addr)
            }else{
                string::utf8(b"")
            };
            vector::push_back(&mut results, author_id);
            i = i + 1;
        };
        results
    }

    public entry fun verify_and_binding_twitter_account(tweet_id: String){
        let tweet_obj_id = tweet_v2::tweet_object_id(tweet_id);
        assert!(tweet_v2::exists_tweet_object(tweet_id), ErrorTweetNotFound);

        let tweet_obj = tweet_v2::take_tweet_object_internal(tweet_obj_id);
        let tweet = object::borrow(&tweet_obj);
        let author_id = *tweet_v2::tweet_author_id(tweet);
        let bitcoin_address = verify_binding_tweet(tweet);
        binding_twitter_account(tweet_obj, author_id, bitcoin_address);
    }

    public fun check_binding_tweet(tweet_id: String): BitcoinAddress {
        let tweet_obj = tweet_v2::borrow_tweet_object(tweet_id);
        let tweet = object::borrow(tweet_obj);
        let author_id = *tweet_v2::tweet_author_id(tweet);
        let btc_address = verify_binding_tweet(tweet);
        let mapping = borrow_twitter_account_mapping();
        assert!(!table::contains(&mapping.account_to_address, author_id), ErrorAccountAlreadyBound);
        btc_address
    }

    fun binding_twitter_account(tweet_obj: Object<Tweet>, author_id: String, bitcoin_address: BitcoinAddress){
        let user_rooch_address = bitcoin_address::to_rooch_address(&bitcoin_address);
        let mapping = borrow_mut_twitter_account_mapping();
        assert!(!table::contains(&mapping.account_to_address, author_id), ErrorAccountAlreadyBound);
        let tweet_obj_id = object::id(&tweet_obj);
        //Transfer the binding tweet object to the user
        tweet_v2::transfer_tweet_object_internal(tweet_obj, user_rooch_address);

        let twitter_account = TwitterAccount{
            id: author_id,
            binding_tweet_obj_id: tweet_obj_id,
        };
 
        let twitter_account_obj = object::new_account_named_object(user_rooch_address, twitter_account);
        object::transfer_extend(twitter_account_obj, user_rooch_address);

        table::add(&mut mapping.account_to_address, author_id, user_rooch_address);
        table::add(&mut mapping.address_to_account, user_rooch_address, author_id); 
        reward_rgas_to_user(author_id, user_rooch_address);
        event::emit(TwitterBindingEvent{
            tweet_obj_id: tweet_obj_id,
            author_id,
            bitcoin_address,
            account_address: user_rooch_address,
        });
    }

    fun reward_rgas_to_user(author_id: String, user_rooch_address: address){
        let faucet = borrow_mut_twitter_rgas_faucet();
        if (!faucet.is_open){
            return
        };
        //One twitter user can only claim once, if the user unbinding and binding again, we will not reward again
        if (table::contains(&faucet.claim_records, author_id)){
            return
        };
        let balance = coin_store::balance(&faucet.rgas_store);
        if (balance >= REWARD_RGAS_AMOUNT){
            let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, REWARD_RGAS_AMOUNT);
            account_coin_store::deposit<RGas>(user_rooch_address, rgas_coin);
            table::add(&mut faucet.claim_records, author_id, REWARD_RGAS_AMOUNT);
        };
    }

    public fun check_user_claimed(author_id: String): bool{
        let faucet = borrow_twitter_rgas_faucet();
        return table::contains(&faucet.claim_records, author_id)
    }

    public entry fun unbinding_twitter_account(owner: &signer){
        let user_rooch_address = signer::address_of(owner);
        unbinding_twitter_account_internal(user_rooch_address);
    }

    fun unbinding_twitter_account_internal(user_rooch_address: address){
        let mapping = borrow_mut_twitter_account_mapping();
        let author_id: String = table::remove(&mut mapping.address_to_account, user_rooch_address);
        table::remove(&mut mapping.account_to_address, author_id);
        let twitter_account_obj_id = object::account_named_object_id<TwitterAccount>(user_rooch_address);
        let twitter_account_obj = object::take_object_extend<TwitterAccount>(twitter_account_obj_id);
        let twitter_account = object::remove(twitter_account_obj);
        let TwitterAccount{
            id:_,
            binding_tweet_obj_id
        } = twitter_account;
        tweet_v2::remove_tweet_object_internal(binding_tweet_obj_id);
    }

    public entry fun claim_tweet(tweet_id: String){
        let tweet_obj_id = tweet_v2::tweet_object_id(tweet_id);
        assert!(tweet_v2::exists_tweet_object(tweet_id), ErrorTweetNotFound);
        let tweet_obj = tweet_v2::take_tweet_object_internal(tweet_obj_id);
        let tweet = object::borrow(&tweet_obj);
        let author_id = *tweet_v2::tweet_author_id(tweet);
        let author_address_opt = resolve_address_by_author_id(author_id);
        assert!(option::is_some(&author_address_opt), ErrorAuthorAddressNotFound);
        let owner_address = option::destroy_some(author_address_opt);
        tweet_v2::transfer_tweet_object_internal(tweet_obj, owner_address);
    }

    public entry fun deposit_rgas_coin(
        account: &signer,
        amount: u256
    ){
        let faucet = borrow_mut_twitter_rgas_faucet();
        deposit_to_rgas_store(account, &mut faucet.rgas_store, amount);
    }

    public entry fun withdraw_rgas_coin( 
        amount: u256,
        _admin: &mut Object<AdminCap>,
    ){
        let faucet = borrow_mut_twitter_rgas_faucet();
        let rgas_coin = coin_store::withdraw(&mut faucet.rgas_store, amount);
        account_coin_store::deposit<RGas>(sender(), rgas_coin);
    }

    public entry fun close_faucet(
        _admin: &mut Object<AdminCap>,
    ){
        let faucet = borrow_mut_twitter_rgas_faucet();
        faucet.is_open = false;
    }

    public entry fun open_faucet(
        _admin: &mut Object<AdminCap>,
    ) {
        let faucet = borrow_mut_twitter_rgas_faucet();
        faucet.is_open = true;
    }

    fun verify_binding_tweet(tweet: &Tweet): BitcoinAddress{
        let text = tweet_v2::tweet_text(tweet);
        let text_bytes = string::bytes(text);
        assert!(starts_with(text_bytes, &TWITTER_ACCOUNT_BINDING_MESSAGE_PREFIX), ErrorTweetBindingMessageInvalidPrefix);
        let entities = tweet_v2::tweet_entities(tweet);
        let hashtags = tweet_v2::tweet_entities_hashtags(entities);
        assert!(!vector::is_empty(hashtags), ErrorTweetBindingMessageMissingHashtag);
        let hashtags_len = vector::length(hashtags);
        let i = 0;
        let found = false;
        while (i < hashtags_len){
            let hashtag = vector::borrow(hashtags, i);
            let tag = tweet_v2::tweet_tag_tag(hashtag);
            if (string::bytes(tag) == &TWITTER_ACCOUNT_BINDING_HASH_TAG){
                found = true;
                break
            };
            i = i + 1;
        };
        assert!(found, ErrorTweetBindingMessageMissingHashtag);
        let bitcoin_address_str = get_bitcoin_address_from_tweet_text(text_bytes);
        assert!(is_valid_bitcoin_address_str(&bitcoin_address_str), ErrorTweetBindingMessageInvalidAddress);
        bitcoin_address::from_string(&string::utf8(bitcoin_address_str))
    }

    fun is_valid_bitcoin_address_str(bitcoin_address_str: &vector<u8>): bool{
        if (vector::length(bitcoin_address_str) < 26){
            return false
        };
        let is_mainnet = bitcoin_move::network::is_mainnet();
        if (is_mainnet && !starts_with(bitcoin_address_str, &BITCOIN_TAPROOT_ADDRESS_PREFIX_MAINNET)){
            return false
        };
        //TODO do more verify for bitcoin address
        true
    }

    fun get_bitcoin_address_from_tweet_text(text: &vector<u8>): vector<u8> {
        let prefix_len = vector::length(&TWITTER_ACCOUNT_BINDING_MESSAGE_PREFIX);
        let split_index = find_split_index(text, prefix_len);
        vector::slice(text, prefix_len, split_index)
    }

    fun find_split_index(text: &vector<u8>, start_index: u64): u64 {
        let i = start_index;
        let text_len = vector::length(text);
        while (i < text_len) {
            if (vector::contains(&ADDRESS_SPLIT_CHARS, vector::borrow(text, i))){
                return i
            };
            i = i + 1;
        };
        text_len
    }

    //TODO migrate to std::string::starts_with
    fun starts_with(haystack: &vector<u8>, needle: &vector<u8>): bool {
        let haystack_len = vector::length(haystack);
        let needle_len = vector::length(needle);

        if (needle_len > haystack_len) {
            return false
        };

        let i = 0;
        while (i < needle_len) {
            if (vector::borrow(haystack, i) != vector::borrow(needle, i)) {
                return false
            };
            i = i + 1;
        };

        true
    }

    fun borrow_twitter_rgas_faucet() : &TwitterRGasFaucet{
        let twitter_rgas_faucet_obj_id = object::named_object_id<TwitterRGasFaucet>();
        let twitter_rgas_faucet_obj = object::borrow_object<TwitterRGasFaucet>(twitter_rgas_faucet_obj_id);
        object::borrow(twitter_rgas_faucet_obj)
    }

    fun borrow_mut_twitter_rgas_faucet() : &mut TwitterRGasFaucet{
        let twitter_rgas_faucet_obj_id = object::named_object_id<TwitterRGasFaucet>();
        let twitter_rgas_faucet_obj = object::borrow_mut_object_extend<TwitterRGasFaucet>(twitter_rgas_faucet_obj_id);
        object::borrow_mut(twitter_rgas_faucet_obj)
    }

    fun borrow_twitter_account_mapping() : &TwitterAccountMapping{
        let mapping_obj_id = object::named_object_id<TwitterAccountMapping>();
        let mapping_obj = object::borrow_object<TwitterAccountMapping>(mapping_obj_id);
        object::borrow(mapping_obj)
    }

    fun borrow_mut_twitter_account_mapping() : &mut TwitterAccountMapping{
        let mapping_obj_id = object::named_object_id<TwitterAccountMapping>();
        let mapping_obj = object::borrow_mut_object_extend<TwitterAccountMapping>(mapping_obj_id);
        object::borrow_mut(mapping_obj)
    }

    fun deposit_to_rgas_store(
        account: &signer,
        rgas_store: &mut Object<CoinStore<RGas>>,
        amount: u256
    ){
        let rgas_coin = account_coin_store::withdraw<RGas>(account, amount);
        coin_store::deposit(rgas_store, rgas_coin);
    }

    // ============================ Test functions ============================

    #[test]
    fun test_verify_binding_tweet(){
        bitcoin_move::genesis::init_for_test();
        let btc_address_str = b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g";
        let tweet_obj = tweet_v2::new_tweet_object_for_test(b"{\"note_tweet\": {\"text\": \"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g #RoochNetwork\",\"entities\": {\"hashtags\": [{\"start\": 0,\"end\": 33,\"tag\": \"RoochNetwork\"}]}},\"author_id\": \"987654321\",\"id\": \"1234567890123456789\",\"text\": \"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g #RoochNetwork\",\"created_at\": \"2024-01-01T00:00:00.000\"}");
        let tweet = object::borrow(&tweet_obj);
        let bitcoin_address = verify_binding_tweet(tweet);
        assert!(bitcoin_address == bitcoin_address::from_string(&string::utf8(btc_address_str)), 2);
        tweet_v2::transfer_tweet_object_internal(tweet_obj, @twitter_binding);
    }

    #[test]
    fun test_verify_and_binding_twitter_account(){
        bitcoin_move::genesis::init_for_test();
        rooch_framework::gas_coin::faucet_for_test(@twitter_binding, INIT_GAS_AMOUNT*2);
        init();
        let btc_address_str = b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g";
        let expect_btc_address = bitcoin_address::from_string(&string::utf8(btc_address_str));
        let expect_owner_address = bitcoin_address::to_rooch_address(&expect_btc_address);
        let tweet_id = string::utf8(b"1234567890123456789");
        let author_id = string::utf8(b"987654321");
        let tweet_obj = tweet_v2::new_tweet_object_for_test(b"{\"note_tweet\": {\"text\": \"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g #RoochNetwork\",\"entities\": {\"hashtags\": [{\"start\": 0,\"end\": 33,\"tag\": \"RoochNetwork\"}]}},\"author_id\": \"987654321\",\"id\": \"1234567890123456789\",\"text\": \"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g #RoochNetwork\",\"created_at\": \"2024-01-01T00:00:00.000\"}");
        tweet_v2::transfer_tweet_object_internal(tweet_obj, @twitter_binding);
        verify_and_binding_twitter_account(tweet_id);
        let author_address_opt = resolve_address_by_author_id(author_id);
        assert!(option::is_some(&author_address_opt), 3);
        let author_address = option::destroy_some(author_address_opt);
        assert!(author_address == expect_owner_address, 4);
        let tweet_obj = tweet_v2::borrow_tweet_object(tweet_id);
        assert!(object::owner(tweet_obj) == expect_owner_address, 5);

        unbinding_twitter_account_internal(expect_owner_address);
        let author_address_opt = resolve_address_by_author_id(author_id);
        assert!(option::is_none(&author_address_opt), 6);
    }

    #[test]
    fun test_get_bitcoin_address_from_tweet_text(){
        let text = b"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g hello";
        let bitcoin_address_str = get_bitcoin_address_from_tweet_text(&text);
        assert!(bitcoin_address_str == b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g", 1);

        let text = b"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g\nabcd";
        let bitcoin_address_str = get_bitcoin_address_from_tweet_text(&text);
        assert!(bitcoin_address_str == b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g", 2);

        let text = b"BTC:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g.";
        let bitcoin_address_str = get_bitcoin_address_from_tweet_text(&text);
        assert!(bitcoin_address_str == b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g", 3);
    }
}
