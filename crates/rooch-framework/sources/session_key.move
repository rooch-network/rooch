module rooch_framework::session_key {
    use std::vector;
    use std::option::{Self, Option};
    use std::error;
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::account_storage;
    use moveos_std::table::{Self, Table};
    use moveos_std::tx_meta::{Self, FunctionCallMeta};
    use rooch_framework::auth_validator;
    use rooch_framework::native_validator;
    use rooch_framework::timestamp;

    friend rooch_framework::transaction_validator;

    /// Create session key in this context is not allowed
    const ErrorSessionKeyCreatePermissionDenied: u64 = 1;
    /// The session key already exists
    const ErrorSessionKeyAlreadyExists: u64 = 2;
    /// The session key is invalid
    const ErrorSessionKeyIsInvalid: u64 = 3;
    /// The session is expired
    const ErrorSessionIsExpired: u64 = 4;
    /// The function call is beyond the session's scope
    const ErrorFunctionCallBeyondSessionScope: u64 = 5;
    /// The lengths of the parts of the session's scope do not match.
    const ErrorSessionScopePartLengthNotMatch: u64 = 6;

    /// The session's scope
    struct SessionScope has store,copy,drop {
        /// The scope module address, the address can not support `*`
        module_address: address,
        /// The scope module name, `*` means all modules in the module address
        module_name: std::ascii::String,
        /// The scope function name, `*` means all functions in the module
        function_name: std::ascii::String,
    }

    struct SessionKey has store,copy,drop {
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

    public fun new_session_scope(module_address: address, module_name: std::ascii::String, function_name: std::ascii::String) : SessionScope {
        SessionScope {
            module_address: module_address,
            module_name: module_name,
            function_name: function_name,
        }
    }

    fun is_expired(ctx: &StorageContext, session_key: &SessionKey) : bool {
        let now_seconds = timestamp::now_seconds(ctx);
        if (session_key.max_inactive_interval > 0 && session_key.last_active_time + session_key.max_inactive_interval < now_seconds){
            return true
        };
        return false
    }

    public fun is_expired_session_key(ctx: &StorageContext, account_address: address, authentication_key: vector<u8>) : bool {
        let session_key_option = get_session_key(ctx, account_address, authentication_key);
        if (option::is_none(&session_key_option)){
            return false
        };
        let session_key = option::extract(&mut session_key_option);
        is_expired(ctx, &session_key)
    }

    public fun exists_session_key(ctx: &StorageContext, account_address: address, authentication_key: vector<u8>) : bool {
        option::is_some(&get_session_key(ctx, account_address, authentication_key))
    }

    /// Get the session key of the account_address by the authentication key
    public fun get_session_key(ctx: &StorageContext, account_address: address, authentication_key: vector<u8>) : Option<SessionKey> {
        if (!account_storage::global_exists<SessionKeys>(ctx, account_address)){
            return option::none()
        };
        let session_keys = account_storage::global_borrow<SessionKeys>(ctx, account_address);
        if (!table::contains(&session_keys.keys, authentication_key)){
            return option::none()
        }else{
            option::some(*table::borrow(&session_keys.keys, authentication_key))
        }
    }

    public fun create_session_key(ctx: &mut StorageContext, sender: &signer, authentication_key: vector<u8>, scopes: vector<SessionScope>, max_inactive_interval: u64) {
        //Can not create new session key by the other session key
        assert!(!auth_validator::is_validate_via_session_key(ctx), error::permission_denied(ErrorSessionKeyCreatePermissionDenied));
        let sender_addr = signer::address_of(sender);
        assert!(!exists_session_key(ctx, sender_addr, authentication_key), error::already_exists(ErrorSessionKeyAlreadyExists));
        let now_seconds = timestamp::now_seconds(ctx);
        let session_key = SessionKey {
            authentication_key: authentication_key,
            scopes: scopes,
            create_time: now_seconds,
            last_active_time: now_seconds,
            max_inactive_interval: max_inactive_interval,
        };
        if (!account_storage::global_exists<SessionKeys>(ctx, sender_addr)){
            let keys = table::new<vector<u8>, SessionKey>(storage_context::tx_context_mut(ctx));
            account_storage::global_move_to<SessionKeys>(ctx, sender, SessionKeys{keys});
        };

        let session_keys = account_storage::global_borrow_mut<SessionKeys>(ctx, sender_addr);
        table::add(&mut session_keys.keys, authentication_key, session_key);
    }

    public entry fun create_session_key_entry(ctx: &mut StorageContext, sender: &signer, authentication_key: vector<u8>, scope_module_address: address, scope_module_name: std::ascii::String, scope_function_name: std::ascii::String, max_inactive_interval: u64) {
        create_session_key(ctx, sender, authentication_key, vector::singleton(SessionScope{
            module_address: scope_module_address,
            module_name: scope_module_name,
            function_name: scope_function_name,
        }), max_inactive_interval);
    }

    public entry fun create_session_key_with_multi_scope_entry(
        ctx: &mut StorageContext, 
        sender: &signer, 
        authentication_key: vector<u8>, 
        scope_module_addresses: vector<address>, 
        scope_module_names: vector<std::ascii::String>, 
        scope_function_names: vector<std::ascii::String>, 
        max_inactive_interval: u64) {
        assert!(
            vector::length<address>(scope_module_addresses) == vector::length<std::ascii::String>(scope_module_names) &&
            vector::length<std::ascii::String>(scope_module_names) == vector::length<std::ascii::String>(scope_function_names),
            error::invalid_argument(ErrorSessionScopePartLengthNotMatch)
        );
        
        let idx = 0;
        let scopes = vector::empty<SessionScope>();

        while(idx < vector::length(&scope_module_addresses)){
            let scope_module_address = vector::borrow(&scope_module_addresses, idx);
            let scope_module_name = vector::borrow(&scope_module_names, idx);
            let scope_function_name = vector::borrow(&scope_function_names, idx);

            vector::push_back(&scopes, SessionScope{
                module_address: scope_module_address,
                module_name: scope_module_name,
                function_name: scope_function_name,
            });
            
            idx = idx + 1;
        };

        create_session_key(ctx, sender, authentication_key, scopes, max_inactive_interval);
    }

    /// Validate the current tx via the session key
    /// If the authentication key is not a session key, return option::none
    /// If the session key is expired or invalid, abort the tx, otherwise return option::some(authentication key)
    public(friend) fun validate(ctx: &StorageContext, auth_validator_id: u64, authenticator_payload: vector<u8>) : Option<vector<u8>> {
        let sender_addr = storage_context::sender(ctx);
        if (!account_storage::global_exists<SessionKeys>(ctx, sender_addr)){
            return option::none()
        };
        // We only support native validator for SessionKey now
        if(auth_validator_id != native_validator::auth_validator_id()){
            return option::none()
        };

        let auth_key = native_validator::get_authentication_key_from_authenticator_payload(&authenticator_payload);
        
        let session_key_option = get_session_key(ctx, sender_addr, auth_key);
        if (option::is_none(&session_key_option)){
            return option::none()
        };
        let session_key = option::extract(&mut session_key_option);
        assert!(!is_expired(ctx, &session_key), error::permission_denied(ErrorSessionIsExpired));
        
        assert!(in_session_scope(ctx, &session_key), error::permission_denied(ErrorFunctionCallBeyondSessionScope));

        native_validator::validate_signature(&authenticator_payload, &storage_context::tx_hash(ctx));
        option::some(auth_key)
    }

    /// Check the current tx is in the session scope or not
    fun in_session_scope(ctx: &StorageContext, session_key: &SessionKey): bool{
        let idx = 0;
        let tx_meta = storage_context::tx_meta(ctx);
        
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

    fun is_asterisk(str: &std::ascii::String) : bool {
        let asterisk = std::ascii::string(b"*");
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

    public(friend) fun active_session_key(ctx: &mut StorageContext, authentication_key: vector<u8>) {
        let sender_addr = storage_context::sender(ctx);
        let now_seconds = timestamp::now_seconds(ctx);
        assert!(account_storage::global_exists<SessionKeys>(ctx, sender_addr), error::not_found(ErrorSessionKeyIsInvalid));
        let session_keys = account_storage::global_borrow_mut<SessionKeys>(ctx, sender_addr);
        assert!(table::contains(&session_keys.keys, authentication_key), error::not_found(ErrorSessionKeyIsInvalid));
        let session_key = table::borrow_mut(&mut session_keys.keys, authentication_key);
        session_key.last_active_time = now_seconds;
    }

    #[test_only]
    public fun active_session_key_for_test(ctx: &mut StorageContext, authentication_key: vector<u8>) {
        active_session_key(ctx, authentication_key);
    }

    public fun remove_session_key(ctx: &mut StorageContext, sender: &signer, authentication_key: vector<u8>) {
        let sender_addr = signer::address_of(sender);
        assert!(account_storage::global_exists<SessionKeys>(ctx, sender_addr), error::not_found(ErrorSessionKeyIsInvalid));
        let session_keys = account_storage::global_borrow_mut<SessionKeys>(ctx, sender_addr);
        assert!(table::contains(&session_keys.keys, authentication_key), error::not_found(ErrorSessionKeyIsInvalid));
        table::remove(&mut session_keys.keys, authentication_key);
    }

    public entry fun remove_session_key_entry(ctx: &mut StorageContext, sender: &signer, authentication_key: vector<u8>) {
        remove_session_key(ctx, sender, authentication_key);
    }

    #[test]
    fun test_check_scope_match() {
        let scope = new_session_scope(@0x1, std::ascii::string(b"test"), std::ascii::string(b"test"));
        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test"), std::ascii::string(b"test"));
        assert!(check_scope_match(&scope, &function_call_meta), 1000);
        
        let function_call_meta = tx_meta::new_function_call_meta(@0x2, std::ascii::string(b"test"), std::ascii::string(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1001);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test1"), std::ascii::string(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1002);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test"), std::ascii::string(b"test1"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1003);
    }

     #[test]
    fun test_check_scope_match_asterisk() {
        let scope = new_session_scope(@0x1, std::ascii::string(b"*"), std::ascii::string(b"*"));
        
        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test"), std::ascii::string(b"test"));
        assert!(check_scope_match(&scope, &function_call_meta), 1000);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test2"), std::ascii::string(b"test2"));
        assert!(check_scope_match(&scope, &function_call_meta), 1001);
        
        let function_call_meta = tx_meta::new_function_call_meta(@0x2, std::ascii::string(b"test"), std::ascii::string(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1002);

        let scope = new_session_scope(@0x1, std::ascii::string(b"test"), std::ascii::string(b"*"));

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test"), std::ascii::string(b"test1"));
        assert!(check_scope_match(&scope, &function_call_meta), 1003);

        let function_call_meta = tx_meta::new_function_call_meta(@0x1, std::ascii::string(b"test1"), std::ascii::string(b"test"));
        assert!(!check_scope_match(&scope, &function_call_meta), 1004);
    }

}