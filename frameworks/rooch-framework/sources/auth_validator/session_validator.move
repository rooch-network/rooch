// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements the session auth validator.
module rooch_framework::session_validator {

    use std::vector;
    use std::option;
    use moveos_std::tx_context;
    use rooch_framework::ed25519;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::ecdsa_r1;
    use rooch_framework::auth_validator;
    use rooch_framework::session_key;

    friend rooch_framework::transaction_validator;

    /// there defines auth validator id for each auth validator
    const SESSION_VALIDATOR_ID: u64 = 0;


    struct SessionValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        SESSION_VALIDATOR_ID
    }

    /// Parse authenticator payload and return (signature, public_key, envelope, optional_message)
    /// Supports both v1 (legacy) and v2 (envelope-aware) formats
    fun parse_authenticator_payload(authenticator_payload: &vector<u8>): (vector<u8>, vector<u8>, u8, option::Option<vector<u8>>) {
        let payload_len = vector::length(authenticator_payload);
        assert!(payload_len > 0, auth_validator::error_validate_invalid_authenticator());
        
        let scheme = *vector::borrow(authenticator_payload, 0);
        
        // Determine if this is v1 or v2 format by checking payload structure
        let (envelope, offset) = detect_format_and_envelope(authenticator_payload, scheme);
        
        // Parse signature and public key based on scheme
        let (signature, public_key, next_offset) = parse_signature_and_key(authenticator_payload, scheme, offset);
        
        // Parse optional message for envelopes that require it
        let message_option = if (envelope == session_key::signing_envelope_bitcoin_message_v0()) {
            parse_message(authenticator_payload, next_offset)
        } else {
            option::none<vector<u8>>()
        };
        
        (signature, public_key, envelope, message_option)
    }

    /// Detect payload format (v1/v2) and extract envelope
    fun detect_format_and_envelope(payload: &vector<u8>, scheme: u8): (u8, u64) {
        let payload_len = vector::length(payload);
        
        // Calculate expected v1 payload length for each scheme
        let expected_v1_len = if (scheme == session_key::signature_scheme_ed25519()) {
            1 + ed25519::signature_length() + ed25519::public_key_length() // 1 + 64 + 32 = 97
        } else if (scheme == session_key::signature_scheme_secp256k1()) {
            1 + 64 + ecdsa_k1::public_key_length() // 1 + 64 + 33 = 98
        } else if (scheme == session_key::signature_scheme_ecdsar1()) {
            1 + ecdsa_r1::raw_signature_length() + ecdsa_r1::public_key_length() // 1 + 64 + 33 = 98
        } else {
            abort auth_validator::error_validate_invalid_authenticator()
        };
        
        if (payload_len == expected_v1_len) {
            // v1 format: implicit RawTxHash envelope
            (session_key::signing_envelope_raw_tx_hash(), 1)
        } else {
            // v2 format: explicit envelope at position 1
            assert!(payload_len > 1, auth_validator::error_validate_invalid_authenticator());
            let envelope = *vector::borrow(payload, 1);
            (envelope, 2)
        }
    }

    /// Parse signature and public key from payload
    fun parse_signature_and_key(payload: &vector<u8>, scheme: u8, offset: u64): (vector<u8>, vector<u8>, u64) {
        let signature = vector::empty<u8>();
        let public_key = vector::empty<u8>();
        
        if (scheme == session_key::signature_scheme_ed25519()) {
            // Parse Ed25519 signature (64 bytes) and public key (32 bytes)
            let sig_len = ed25519::signature_length();
            let pk_len = ed25519::public_key_length();
            
            let i = offset;
            while (i < offset + sig_len) {
                vector::push_back(&mut signature, *vector::borrow(payload, i));
                i = i + 1;
            };
            
            let i = offset + sig_len;
            while (i < offset + sig_len + pk_len) {
                vector::push_back(&mut public_key, *vector::borrow(payload, i));
                i = i + 1;
            };
            
            (signature, public_key, offset + sig_len + pk_len)
        } else if (scheme == session_key::signature_scheme_secp256k1()) {
            // Parse Secp256k1 signature (64 bytes) and public key (33 bytes)
            let sig_len = 64;
            let pk_len = ecdsa_k1::public_key_length();
            
            let i = offset;
            while (i < offset + sig_len) {
                vector::push_back(&mut signature, *vector::borrow(payload, i));
                i = i + 1;
            };
            
            let i = offset + sig_len;
            while (i < offset + sig_len + pk_len) {
                vector::push_back(&mut public_key, *vector::borrow(payload, i));
                i = i + 1;
            };
            
            (signature, public_key, offset + sig_len + pk_len)
        } else if (scheme == session_key::signature_scheme_ecdsar1()) {
            // Parse Secp256r1 signature (64 bytes) and public key (33 bytes)
            let sig_len = ecdsa_r1::raw_signature_length();
            let pk_len = ecdsa_r1::public_key_length();
            
            let i = offset;
            while (i < offset + sig_len) {
                vector::push_back(&mut signature, *vector::borrow(payload, i));
                i = i + 1;
            };
            
            let i = offset + sig_len;
            while (i < offset + sig_len + pk_len) {
                vector::push_back(&mut public_key, *vector::borrow(payload, i));
                i = i + 1;
            };
            
            (signature, public_key, offset + sig_len + pk_len)
        } else {
            abort auth_validator::error_validate_invalid_authenticator()
        }
    }

    /// Parse message from payload (for envelopes that require it)
    fun parse_message(payload: &vector<u8>, offset: u64): option::Option<vector<u8>> {
        let payload_len = vector::length(payload);
        if (offset >= payload_len) {
            return option::none<vector<u8>>()
        };
        
        // Read message length (VarInt, single-byte path only)
        let msg_len = (*vector::borrow(payload, offset) as u64);
        let msg_start = offset + 1;
        let msg_end = msg_start + msg_len;
        
        assert!(msg_end <= payload_len, auth_validator::error_validate_invalid_authenticator());
        
        let message = vector::empty<u8>();
        let i = msg_start;
        while (i < msg_end) {
            vector::push_back(&mut message, *vector::borrow(payload, i));
            i = i + 1;
        };
        
        option::some(message)
    }

    // validate the signature of the authenticator payload and return auth key
    fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>) : vector<u8> {
        let (signature, public_key, envelope, message_option) = parse_authenticator_payload(authenticator_payload);
        let scheme = *vector::borrow(authenticator_payload, 0);
        
        // Compute digest based on envelope
        let digest = compute_digest_for_envelope(envelope, tx_hash, &message_option);
        
        if (scheme == session_key::signature_scheme_ed25519()) {
            // Ed25519 verification
            assert!(
                ed25519::verify(
                    &signature,
                    &public_key,
                    &digest
                ),
                auth_validator::error_validate_invalid_authenticator()
            );
            session_key::ed25519_public_key_to_authentication_key(&public_key)
        } else if (scheme == session_key::signature_scheme_secp256k1()) {
            // Secp256k1 verification
            assert!(
                ecdsa_k1::verify(
                    &signature,
                    &public_key,
                    &digest,
                    ecdsa_k1::sha256()
                ),
                auth_validator::error_validate_invalid_authenticator()
            );
            session_key::secp256k1_public_key_to_authentication_key(&public_key)
        } else if (scheme == session_key::signature_scheme_ecdsar1()) {
            // Secp256r1 verification
            assert!(
                ecdsa_r1::verify(
                    &signature,
                    &public_key,
                    &digest
                ),
                auth_validator::error_validate_invalid_authenticator()
            );
            session_key::secp256r1_public_key_to_authentication_key(&public_key)
        } else {
            // This should not happen as parse_authenticator_payload already checks
            abort auth_validator::error_validate_invalid_authenticator()
        }
    }

    /// Compute digest based on envelope type
    fun compute_digest_for_envelope(envelope: u8, tx_hash: &vector<u8>, message_option: &option::Option<vector<u8>>): vector<u8> {
        if (envelope == session_key::signing_envelope_raw_tx_hash()) {
            // RawTxHash: digest = tx_hash
            *tx_hash
        } else if (envelope == session_key::signing_envelope_bitcoin_message_v0()) {
            // BitcoinMessageV0: verify message matches canonical template, then compute Bitcoin digest
            assert!(option::is_some(message_option), auth_validator::error_validate_invalid_authenticator());
            let message = option::borrow(message_option);
            
            // Verify message matches canonical template
            let expected_template = session_key::build_canonical_template(tx_hash);
            assert!(*message == expected_template, auth_validator::error_validate_invalid_authenticator());
            
            // Compute Bitcoin message digest
            session_key::bitcoin_message_digest(message)
        } else {
            // Unknown envelope
            abort auth_validator::error_validate_invalid_authenticator()
        }
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
}
