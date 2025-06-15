// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::session_key {
    use std::vector;
    use std::option::{Self, Option};
    use std::signer;
    use moveos_std::object::ObjectID;
    use moveos_std::account;
    use moveos_std::tx_context; 
    use moveos_std::table::{Self, Table};
    use moveos_std::tx_meta::{Self, FunctionCallMeta};
    use rooch_framework::auth_validator;
    use moveos_std::timestamp;
    use moveos_std::hash;

    friend rooch_framework::transaction_validator;
    friend rooch_framework::session_validator;
    friend rooch_framework::did;

    const MAX_INACTIVE_INTERVAL: u64 = 3600 * 24 * 365; // 1 year
    public fun max_inactive_interval(): u64 {
        MAX_INACTIVE_INTERVAL
    }

    /// Create session key in this context is not allowed
    const ErrorSessionKeyCreatePermissionDenied: u64 = 1;
    /// The session key already exists
    const ErrorSessionKeyAlreadyExists: u64 = 2;
    /// The session key is invalid
    const ErrorSessionKeyIsInvalid: u64 = 3;
    /// The lengths of the parts of the session's scope do not match.
    const ErrorSessionScopePartLengthNotMatch: u64 = 4;
    /// The max inactive interval is invalid
    const ErrorInvalidMaxInactiveInterval: u64 = 5;

    // Signature scheme constant, similar to session_validator.move
    const SIGNATURE_SCHEME_ED25519: u8 = 0;
    const SIGNATURE_SCHEME_SECP256K1: u8 = 1;
    const SIGNATURE_SCHEME_ECDSAR1: u8 = 2;
    
    public fun signature_scheme_ed25519(): u8 {
        SIGNATURE_SCHEME_ED25519
    }

    public fun signature_scheme_secp256k1(): u8 {
        SIGNATURE_SCHEME_SECP256K1
    }

    public fun signature_scheme_ecdsar1(): u8 {
        SIGNATURE_SCHEME_ECDSAR1
    }

    /// The session's scope
    struct SessionScope has store,copy,drop {
        /// The scope module address, the address can not support `*`
        module_address: address,
        /// The scope module name, `*` means all modules in the module address
        module_name: std::string::String,
        /// The scope function name, `*` means all functions in the module
        function_name: std::string::String,
    }

    struct SessionKey has store,copy,drop {
        /// App name
        app_name: std::string::String,
        /// app website url
        app_url: std::string::String,
        /// The session key's authentication key, it also is the session key's id
        authentication_key: vector<u8>,
        /// The session key's scopes
        scopes: vector<SessionScope>,
        /// The session key's create time, current timestamp in seconds
        create_time: u64,
        /// The session key's last active time, in seconds
        last_active_time: u64,
        /// The session key's max inactive time period, in seconds
        /// If the session key is not active in this time period, it will be expired
        /// If the max_inactive_interval is 0, the session key will never be expired
        max_inactive_interval: u64,
    }

    struct SessionKeys has key {
        keys: Table<vector<u8>, SessionKey>,
    }

    public fun new_session_scope(module_address: address, module_name: std::string::String, function_name: std::string::String) : SessionScope {
        SessionScope {
            module_address: module_address,
            module_name: module_name,
            function_name: function_name,
        }
    }

    public(friend) fun is_expired(session_key: &SessionKey) : bool {
        let now_seconds = timestamp::now_seconds();
        if (session_key.max_inactive_interval > 0 && session_key.last_active_time + session_key.max_inactive_interval < now_seconds){
            return true
        };
        return false
    }

    public fun is_expired_session_key(account_address: address, authentication_key: vector<u8>) : bool {
        let session_key_option = get_session_key(account_address, authentication_key);
        if (option::is_none(&session_key_option)){
            return true
        };

        let session_key = option::extract(&mut session_key_option);
        is_expired(&session_key)
    }

    public fun has_session_key(account_address: address) : bool {
        account::exists_resource<SessionKeys>(account_address)
    }

    public fun exists_session_key(account_address: address, authentication_key: vector<u8>) : bool {
        option::is_some(&get_session_key(account_address, authentication_key))
    }

    /// Get the session key of the account_address by the authentication key
    public fun get_session_key(account_address: address, authentication_key: vector<u8>) : Option<SessionKey> {
        if (!account::exists_resource<SessionKeys>(account_address)){
            return option::none()
        };
        let session_keys = account::borrow_resource<SessionKeys>(account_address);
        if (!table::contains(&session_keys.keys, authentication_key)){
            return option::none()
        }else{
            option::some(*table::borrow(&session_keys.keys, authentication_key))
        }
    }

    public fun create_session_key(
        sender: &signer,
        app_name: std::string::String,
        app_url: std::string::String,
        authentication_key: vector<u8>,
        scopes: vector<SessionScope>,
        max_inactive_interval: u64) {

        //Can not create new session key by the other session key
        assert!(!auth_validator::is_validate_via_session_key(), ErrorSessionKeyCreatePermissionDenied);
        create_session_key_internal(sender, app_name, app_url, authentication_key, scopes, max_inactive_interval);
    }

    /// Create session key internal, it is used to create session key for DID document
    /// It is allowed to create session key by the other session key
    public(friend) fun create_session_key_internal(
        sender: &signer,
        app_name: std::string::String,
        app_url: std::string::String,
        authentication_key: vector<u8>,
        scopes: vector<SessionScope>,
        max_inactive_interval: u64) {

        assert!(max_inactive_interval <= MAX_INACTIVE_INTERVAL, ErrorInvalidMaxInactiveInterval);

        let sender_addr = signer::address_of(sender);
        assert!(!exists_session_key(sender_addr, authentication_key), ErrorSessionKeyAlreadyExists);
        let now_seconds = timestamp::now_seconds();
        let session_key = SessionKey {
            app_name,
            app_url,
            authentication_key,
            scopes,
            create_time: now_seconds,
            last_active_time: now_seconds,
            max_inactive_interval,
        };
        if (!account::exists_resource<SessionKeys>(sender_addr)){
            let keys = table::new<vector<u8>, SessionKey>();
            account::move_resource_to<SessionKeys>(sender, SessionKeys{keys});
        };

        let session_keys = account::borrow_mut_resource<SessionKeys>(sender_addr);
        table::add(&mut session_keys.keys, authentication_key, session_key);
    }

    public entry fun create_session_key_entry(
        sender: &signer,
        app_name: std::string::String,
        app_url: std::string::String,
        authentication_key: vector<u8>,
        scope_module_address: address,
        scope_module_name: std::string::String,
        scope_function_name: std::string::String,
        max_inactive_interval: u64) {
        create_session_key(sender, app_name, app_url, authentication_key, vector::singleton(SessionScope{
            module_address: scope_module_address,
            module_name: scope_module_name,
            function_name: scope_function_name,
        }), max_inactive_interval);
    }

    public entry fun create_session_key_with_multi_scope_entry(
        sender: &signer,
        app_name: std::string::String,
        app_url: std::string::String,
        authentication_key: vector<u8>, 
        scope_module_addresses: vector<address>, 
        scope_module_names: vector<std::string::String>, 
        scope_function_names: vector<std::string::String>, 
        max_inactive_interval: u64) {
        assert!(
            vector::length<address>(&scope_module_addresses) == vector::length<std::string::String>(&scope_module_names) &&
            vector::length<std::string::String>(&scope_module_names) == vector::length<std::string::String>(&scope_function_names),
            ErrorSessionScopePartLengthNotMatch
        );
        
        let idx = 0;
        let scopes = vector::empty<SessionScope>();

        while(idx < vector::length(&scope_module_addresses)){
            let scope_module_address = vector::borrow(&scope_module_addresses, idx);
            let scope_module_name = vector::borrow(&scope_module_names, idx);
            let scope_function_name = vector::borrow(&scope_function_names, idx);

            vector::push_back(&mut scopes, SessionScope{
                module_address: *scope_module_address,
                module_name: *scope_module_name,
                function_name: *scope_function_name,
            });
            
            idx = idx + 1;
        };

        create_session_key(sender, app_name, app_url, authentication_key, scopes, max_inactive_interval);
    }

    /// Check the current tx is in the session scope or not
    public(friend) fun in_session_scope(session_key: &SessionKey): bool{
        let idx = 0;
        let tx_meta = tx_context::tx_meta();
        
        let function_call_meta_option = tx_meta::function_meta(&tx_meta);
        // session key can not be used to execute script or publish module
        // only support function call now
        if (option::is_none(&function_call_meta_option)){
            return false
        };
        let function_call_meta = option::extract(&mut function_call_meta_option);
        while(idx < vector::length(&session_key.scopes)){
            let scope = vector::borrow(&session_key.scopes, idx);
            if(check_scope_match(scope, &function_call_meta)){
                return true
            };
            idx = idx + 1;
        };
        false
    }

    fun is_asterisk(str: &std::string::String) : bool {
        let asterisk = std::string::utf8(b"*");
        str == &asterisk
    }

    fun check_scope_match(scope: &SessionScope, function_call_meta:&FunctionCallMeta) : bool {
        if (&scope.module_address != tx_meta::function_meta_module_address(function_call_meta)){
            return false
        };
        if (!is_asterisk(&scope.module_name) && &scope.module_name != tx_meta::function_meta_module_name(function_call_meta)){
            return false
        };
        if (!is_asterisk(&scope.function_name) && &scope.function_name != tx_meta::function_meta_function_name(function_call_meta)){
            return false
        };
        true
    }

    public(friend) fun active_session_key(authentication_key: vector<u8>) {
        let sender_addr = tx_context::sender();
        let now_seconds = timestamp::now_seconds();
        // If the session key is not exists, do nothing
        // Because the user may remove the session key in the same transaction
        if(!account::exists_resource<SessionKeys>(sender_addr)){
            return
        };
        let session_keys = account::borrow_mut_resource<SessionKeys>(sender_addr);
        if(!table::contains(&session_keys.keys, authentication_key)){
            return
        };
        let session_key = table::borrow_mut(&mut session_keys.keys, authentication_key);
        session_key.last_active_time = now_seconds;
    }

    #[test_only]
    public fun active_session_key_for_test(authentication_key: vector<u8>) {
        active_session_key(authentication_key);
    }
    
    public fun contains_session_key(sender_addr: address, authentication_key: vector<u8>) : bool {
        if(!account::exists_resource<SessionKeys>(sender_addr)){
            return false
        };
        let session_keys = account::borrow_resource<SessionKeys>(sender_addr);
        table::contains(&session_keys.keys, authentication_key)
    }

    public fun remove_session_key(sender: &signer, authentication_key: vector<u8>) {
        let sender_addr = signer::address_of(sender);
        assert!(account::exists_resource<SessionKeys>(sender_addr), ErrorSessionKeyIsInvalid);
        let session_keys = account::borrow_mut_resource<SessionKeys>(sender_addr);
        // If the session key is not exists, do nothing
        if (table::contains(&session_keys.keys, authentication_key)){
            table::remove(&mut session_keys.keys, authentication_key);
        }
    }

    public entry fun remove_session_key_entry(sender: &signer, authentication_key: vector<u8>) {
        remove_session_key(sender, authentication_key);
    }

    public fun get_session_keys_handle(account_address: address) : Option<ObjectID> {
        if (!account::exists_resource<SessionKeys>(account_address)){
            return option::none()
        };
        let session_keys = account::borrow_resource<SessionKeys>(account_address);
        option::some(table::handle(&session_keys.keys))
    }

    #[test]
    fun test_check_scope_match() {
        let scope = new_session_scope(@0x1, std::string::utf8(b"test"), std::string::utf8(b"test"));
        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test"), std::string::utf8(b"test"));
        assert!(check_scope_match(&scope, &function_call_meta), 1000);
        
        let function_call_meta = tx_meta::new_function_call_meta(@0x2, std::string::utf8(b"test"), std::string::utf8(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1001);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test1"), std::string::utf8(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1002);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test"), std::string::utf8(b"test1"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1003);
    }

     #[test]
    fun test_check_scope_match_asterisk() {
        let scope = new_session_scope(@0x1, std::string::utf8(b"*"), std::string::utf8(b"*"));
        
        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test"), std::string::utf8(b"test"));
        assert!(check_scope_match(&scope, &function_call_meta), 1000);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test2"), std::string::utf8(b"test2"));
        assert!(check_scope_match(&scope, &function_call_meta), 1001);
        
        let function_call_meta = tx_meta::new_function_call_meta(@0x2, std::string::utf8(b"test"), std::string::utf8(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1002);

        let scope = new_session_scope(@0x1, std::string::utf8(b"test"), std::string::utf8(b"*"));

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test"), std::string::utf8(b"test1"));
        assert!(check_scope_match(&scope, &function_call_meta), 1003);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::string::utf8(b"test1"), std::string::utf8(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1004);
    }

    /// Derives the authentication key for an Ed25519 public key.
    /// This is consistent with how session_validator derives it.
    public fun ed25519_public_key_to_authentication_key(public_key: &vector<u8>): vector<u8> {
        let bytes_for_hash = vector::singleton(SIGNATURE_SCHEME_ED25519);
        vector::append(&mut bytes_for_hash, *public_key);
        hash::blake2b256(&bytes_for_hash)
    }

    /// Derives the authentication key for a Secp256k1 public key.
    /// This follows the same pattern as Ed25519 but with a different scheme identifier.
    public fun secp256k1_public_key_to_authentication_key(public_key: &vector<u8>): vector<u8> {
        let bytes_for_hash = vector::singleton(SIGNATURE_SCHEME_SECP256K1);
        vector::append(&mut bytes_for_hash, *public_key);
        hash::blake2b256(&bytes_for_hash)
    }

    /// Derives the authentication key for a Secp256r1 public key.
    /// This follows the same pattern as Ed25519 but with a different scheme identifier.
    public fun secp256r1_public_key_to_authentication_key(public_key: &vector<u8>): vector<u8> {
        let auth_key = vector::empty<u8>();
        vector::append(&mut auth_key, vector::singleton(SIGNATURE_SCHEME_ECDSAR1));
        vector::append(&mut auth_key, hash::sha2_256(*public_key));
        auth_key
    }

}
