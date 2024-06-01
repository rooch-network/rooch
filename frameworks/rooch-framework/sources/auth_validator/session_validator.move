// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements the session auth validator.
module rooch_framework::session_validator {

    use std::vector;
    use std::option;
    use moveos_std::tx_context;
    use moveos_std::hash;
    use rooch_framework::ed25519;
    use rooch_framework::auth_validator;
    use rooch_framework::session_key;

    friend rooch_framework::transaction_validator;

    /// there defines auth validator id for each auth validator
    const SESSION_VALIDATOR_ID: u64 = 0;

    const SIGNATURE_SCHEME_ED25519: u8 = 0;


    struct SessionValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        SESSION_VALIDATOR_ID
    }

    /// Validate the authenticator payload, return public key and signature
    fun validate_authenticator_payload(authenticator_payload: &vector<u8>): (vector<u8>, vector<u8>) {
        let scheme = vector::borrow(authenticator_payload, 0);
        assert!(*scheme == SIGNATURE_SCHEME_ED25519, auth_validator::error_validate_invalid_authenticator());

        let sign = vector::empty<u8>();
        let i = 1;
        let signature_position = ed25519::signature_length() + 1;
        while (i < signature_position) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut sign, *value);
            i = i + 1;
        };

        let public_key = vector::empty<u8>();
        let i = 1 + ed25519::signature_length();
        let public_key_position = 1 + ed25519::signature_length() + ed25519::public_key_length();
        while (i < public_key_position) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut public_key, *value);
            i = i + 1;
        };
        (sign, public_key)
    }

    /// Get the authentication key of the given public key.
    fun public_key_to_authentication_key(signature_scheme: u8, public_key: vector<u8>): vector<u8> {
        let bytes = vector::singleton(signature_scheme);
        vector::append(&mut bytes, public_key);
        hash::blake2b256(&bytes)
    }


    // validate the signature of the authenticator payload and return auth key
    fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>) : vector<u8> {
        let (signature, public_key) = validate_authenticator_payload(authenticator_payload);
        assert!(
            ed25519::verify(
                &signature,
                &public_key,
                tx_hash
            ),
            auth_validator::error_validate_invalid_authenticator()
        );
        public_key_to_authentication_key(SIGNATURE_SCHEME_ED25519, public_key)
    }

    public(friend) fun validate(authenticator_payload: vector<u8>) :vector<u8> {
        
        let sender_addr = tx_context::sender();
        assert!(session_key::has_session_key(sender_addr), auth_validator::error_validate_invalid_account_auth_key());
        
        let tx_hash = tx_context::tx_hash();
        let auth_key = validate_signature(&authenticator_payload, &tx_hash);

        let session_key_option = session_key::get_session_key(sender_addr, auth_key);
        assert!(option::is_some(&session_key_option), auth_validator::error_validate_invalid_account_auth_key());
        
        let session_key = option::extract(&mut session_key_option);
        assert!(!session_key::is_expired(&session_key), auth_validator::error_validate_session_is_expired());
        
        assert!(session_key::in_session_scope(&session_key), auth_validator::error_validate_function_call_beyond_session_scope());
        auth_key
    }

    fun pre_execute() {}

    fun post_execute() {}

}
