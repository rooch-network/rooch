module rooch_framework::session_key {
    use std::vector;
    use std::option::{Self, Option};
    use std::error;
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::account_storage;
    use moveos_std::table::{Self, Table};
    use rooch_framework::auth_validator;
    use rooch_framework::ed25519_validator;
    // use rooch_framework::multi_ed25519_validator;
    // use rooch_framework::ecdsa_validator;
    // use rooch_framework::schnorr_validator;

    friend rooch_framework::transaction_validator;

    const ESessionKeyCreatePermissionDenied: u64 = 1;
    const ESessionKeyAlreadyExists: u64 = 2;
    const ESessionKeyIsInvalid: u64 = 3;
    const ESessionIsExpired: u64 = 4;

    /// The session's scope
    struct SessionScope has store,copy,drop {
        //TODO should we allow the scope module address is `*`?
        module_address: address,
        /// The scope module name, `*` means all modules in the module address
        module_name: std::ascii::String,
        /// The scope function name, `*` means all functions in the module
        function_name: std::ascii::String,
    }

    struct SessionKey has store,copy,drop {
        authentication_key: vector<u8>,
        scheme: u64,
        scopes: vector<SessionScope>,
        /// The session key's expiration time period, in seconds, 0 means never expired
        expiration_time: u64,
        /// The session key's last active time
        last_active_time: u64,
        /// The session key's max inactive time period, in seconds
        max_inactive_interval: u64,
    }

    struct SessionKeys has key {
        keys: Table<vector<u8>, SessionKey>,
    }

    public fun is_expired(_ctx: &StorageContext, _session_key: &SessionKey) : bool {
        //TODO check the session key is expired or not after the timestamp is supported
        return false
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

    public fun create_session_key(ctx: &mut StorageContext, sender: &signer, authentication_key: vector<u8>, scheme: u64, scopes: vector<SessionScope>, expiration_time: u64, max_inactive_interval: u64) {
        //Can not create new session key by the other session key
        assert!(!auth_validator::is_validate_via_session_key(ctx), error::permission_denied(ESessionKeyCreatePermissionDenied));
        let sender_addr = signer::address_of(sender);
        assert!(!exists_session_key(ctx, sender_addr, authentication_key), error::already_exists(ESessionKeyAlreadyExists));

        let session_key = SessionKey {
            authentication_key: authentication_key,
            scheme: scheme,
            scopes: scopes,
            expiration_time: expiration_time,
            //TODO set the last active time to now
            last_active_time: 0,
            max_inactive_interval: max_inactive_interval,
        };
        if (!account_storage::global_exists<SessionKeys>(ctx, sender_addr)){
            let keys = table::new<vector<u8>, SessionKey>(storage_context::tx_context_mut(ctx));
            account_storage::global_move_to<SessionKeys>(ctx, sender, SessionKeys{keys});
        };

        let session_keys = account_storage::global_borrow_mut<SessionKeys>(ctx, sender_addr);
        table::add(&mut session_keys.keys, authentication_key, session_key);
    }

    public entry fun create_session_key_entry(ctx: &mut StorageContext, sender: &signer, authentication_key: vector<u8>, scheme: u64, scope_module_address: address, scope_module_name: std::ascii::String, scope_function_name: std::ascii::String,expiration_time: u64, max_inactive_interval: u64) {
        create_session_key(ctx, sender, authentication_key, scheme, vector::singleton(SessionScope{
            module_address: scope_module_address,
            module_name: scope_module_name,
            function_name: scope_function_name,
        }), expiration_time, max_inactive_interval);
    }

    /// Validate the current tx via the session key
    /// If the authentication key is not a session key, return option::none
    /// If the session key is expired or invalid, abort the tx, otherwise return option::some(authentication key)
    public(friend) fun validate(ctx: &StorageContext, scheme: u64, authenticator_payload: vector<u8>) : Option<vector<u8>> {
        let sender_addr = storage_context::sender(ctx);
        if (!account_storage::global_exists<SessionKeys>(ctx, sender_addr)){
            return option::none()
        };
        let auth_key = if(scheme == ed25519_validator::scheme()){
            ed25519_validator::get_authentication_key_from_payload(&authenticator_payload)
        }else{
            //TODO support other built-in validators
            return option::none()
        };
        
        let session_key_option = get_session_key(ctx, sender_addr, auth_key);
        if (option::is_none(&session_key_option)){
            return option::none()
        };
        let session_key = option::extract(&mut session_key_option);
        assert!(!is_expired(ctx, &session_key), error::permission_denied(ESessionIsExpired));
        assert!(session_key.scheme == scheme, error::invalid_argument(ESessionKeyIsInvalid));
        //TODO validate session scopes

        if(scheme == ed25519_validator::scheme()){
            ed25519_validator::validate_signature(&authenticator_payload, &storage_context::tx_hash(ctx));
        }else{ 
            //TODO support other built-in validators
            abort 1
        };
        option::some(auth_key)
    }

    public(friend) fun active_session_key(ctx: &mut StorageContext, authentication_key: vector<u8>) {
        let sender_addr = storage_context::sender(ctx);
        assert!(account_storage::global_exists<SessionKeys>(ctx, sender_addr), error::not_found(ESessionKeyIsInvalid));
        let session_keys = account_storage::global_borrow_mut<SessionKeys>(ctx, sender_addr);
        assert!(table::contains(&session_keys.keys, authentication_key), error::not_found(ESessionKeyIsInvalid));
        let session_key = table::borrow_mut(&mut session_keys.keys, authentication_key);
        //TODO set the last active time to now when the timestamp is supported
        session_key.last_active_time = session_key.last_active_time + 1;
    }
}