// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements the DID auth validator.
/// It enables direct authentication using DID Document verification methods
/// without requiring intermediate session key creation.
module rooch_framework::did_validator {

    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use moveos_std::tx_context;
    use rooch_framework::auth_validator;
    use rooch_framework::did::{Self, DID};
    use rooch_framework::session_key;
    use rooch_framework::ed25519;
    use rooch_framework::ecdsa_r1;
    use moveos_std::hash;
    use moveos_std::json;
    use moveos_std::bcs;
    use moveos_std::base64;

    friend rooch_framework::transaction_validator;
    
    #[test_only]
    friend rooch_framework::did_validator_test;

    /// DID auth validator ID
    const DID_VALIDATOR_ID: u64 = 4;

    /// Envelope types (same as session validator)
    const ENVELOPE_RAW_TX_HASH: u8 = 0x00;
    const ENVELOPE_BITCOIN_MESSAGE_V0: u8 = 0x01;
    const ENVELOPE_WEBAUTHN_V0: u8 = 0x02;

    struct DIDValidator has store, drop {}

    #[data_struct]
    struct DIDAuthPayload has copy, store, drop {
        scheme: u8,
        envelope: u8,
        vm_fragment: String,
        signature: vector<u8>,
        message: Option<vector<u8>>,
    }

    #[data_struct]
    /// WebAuthn envelope data (only WebAuthn-specific fields)
    struct WebauthnEnvelopeData has copy, store, drop {
        authenticator_data: vector<u8>,
        client_data_json: vector<u8>,
    }

    #[data_struct]
    struct ClientData has copy, store, drop {
        challenge: string::String,
        origin: string::String,
        type: string::String,
    }

    public fun auth_validator_id(): u64 {
        DID_VALIDATOR_ID
    }

    /// Parse DID authenticator payload
    fun parse_did_auth_payload(authenticator_payload: &vector<u8>): DIDAuthPayload {
        let payload_len = vector::length(authenticator_payload);
        assert!(payload_len >= 2, auth_validator::error_validate_invalid_authenticator());

        let scheme = *vector::borrow(authenticator_payload, 0);
        
        // Extract envelope (DID validator always requires explicit envelope)
        let (envelope, offset) = extract_envelope(authenticator_payload);
        
        // Parse verification method fragment
        let (vm_fragment, new_offset) = parse_vm_fragment(authenticator_payload, offset);
        
        // Parse signature and optional message
        let (signature, message) = parse_signature_and_message(
            authenticator_payload, 
            scheme, 
            envelope, 
            new_offset
        );

        DIDAuthPayload {
            scheme,
            envelope,
            vm_fragment,
            signature,
            message,
        }
    }

    /// Extract envelope from payload (DID validator always requires explicit envelope)
    fun extract_envelope(payload: &vector<u8>): (u8, u64) {
        let payload_len = vector::length(payload);
        assert!(payload_len >= 2, auth_validator::error_validate_invalid_authenticator());
        
        let envelope = *vector::borrow(payload, 1);
        
        // Validate envelope type
        assert!(
            envelope == ENVELOPE_RAW_TX_HASH || 
            envelope == ENVELOPE_BITCOIN_MESSAGE_V0 || 
            envelope == ENVELOPE_WEBAUTHN_V0,
            auth_validator::error_validate_invalid_authenticator()
        );
        
        (envelope, 2)
    }

    /// Parse verification method fragment from payload
    fun parse_vm_fragment(payload: &vector<u8>, offset: u64): (String, u64) {
        let payload_len = vector::length(payload);
        assert!(offset < payload_len, auth_validator::error_validate_invalid_authenticator());

        let fragment_len = *vector::borrow(payload, offset);
        let new_offset = offset + 1;
        
        assert!(new_offset + (fragment_len as u64) <= payload_len, 
                auth_validator::error_validate_invalid_authenticator());

        let fragment_bytes = vector::empty<u8>();
        let i = 0;
        while (i < fragment_len) {
            vector::push_back(&mut fragment_bytes, *vector::borrow(payload, new_offset + (i as u64)));
            i = i + 1;
        };

        let fragment = string::utf8(fragment_bytes);
        (fragment, new_offset + (fragment_len as u64))
    }

    /// Parse signature and optional message from payload
    fun parse_signature_and_message(
        payload: &vector<u8>, 
        scheme: u8, 
        envelope: u8, 
        offset: u64
    ): (vector<u8>, Option<vector<u8>>) {
        let payload_len = vector::length(payload);
        
        // Determine signature length based on scheme
        let sig_len = if (scheme == session_key::signature_scheme_ed25519()) {
            ed25519::signature_length()
        } else if (scheme == session_key::signature_scheme_secp256k1()) {
            64 // ECDSA signature length
        } else if (scheme == session_key::signature_scheme_ecdsar1()) {
            ecdsa_r1::raw_signature_length()
        } else {
            abort auth_validator::error_validate_invalid_authenticator()
        };

        assert!(offset + sig_len <= payload_len, auth_validator::error_validate_invalid_authenticator());

        // Extract signature
        let signature = vector::empty<u8>();
        let i = 0;
        while (i < sig_len) {
            vector::push_back(&mut signature, *vector::borrow(payload, offset + (i as u64)));
            i = i + 1;
        };

        let new_offset = offset + sig_len;

        // Extract optional message for certain envelope types
        let message = if (envelope == ENVELOPE_BITCOIN_MESSAGE_V0 || envelope == ENVELOPE_WEBAUTHN_V0) {
            if (new_offset < payload_len) {
                let message_len = *vector::borrow(payload, new_offset);
                let msg_start = new_offset + 1;
                
                assert!(msg_start + (message_len as u64) <= payload_len, 
                        auth_validator::error_validate_invalid_authenticator());

                let msg_bytes = vector::empty<u8>();
                let j = 0;
                while (j < message_len) {
                    vector::push_back(&mut msg_bytes, *vector::borrow(payload, msg_start + (j as u64)));
                    j = j + 1;
                };
                option::some(msg_bytes)
            } else {
                option::none()
            }
        } else {
            option::none()
        };

        (signature, message)
    }

    /// Compute digest based on envelope type (reuse session validator logic)
    fun compute_digest(tx_hash: &vector<u8>, envelope: u8, message_option: &Option<vector<u8>>): vector<u8> {
        if (envelope == ENVELOPE_RAW_TX_HASH) {
            // RawTxHash: digest = tx_hash
            *tx_hash
        } else if (envelope == ENVELOPE_BITCOIN_MESSAGE_V0) {
            // BitcoinMessageV0: verify message matches canonical template, then compute Bitcoin digest
            assert!(option::is_some(message_option), auth_validator::error_validate_invalid_authenticator());
            let message = option::borrow(message_option);
            
            // Verify message matches canonical template
            let expected_template = session_key::build_canonical_template(tx_hash);
            assert!(*message == expected_template, auth_validator::error_validate_invalid_authenticator());
            
            // Compute Bitcoin message digest
            session_key::bitcoin_message_digest(message)
        } else if (envelope == ENVELOPE_WEBAUTHN_V0) {
            // WebAuthn: reconstruct message as authenticator_data || SHA256(client_data_json)
            assert!(option::is_some(message_option), auth_validator::error_validate_invalid_authenticator());
            let webauthn_payload_bytes = option::borrow(message_option);
            
            // Compute WebAuthn digest (same logic as session_validator)
            compute_webauthn_digest_from_bcs(webauthn_payload_bytes, tx_hash)
        } else {
            // Unknown envelope
            abort auth_validator::error_validate_invalid_authenticator()
        }
    }

    /// Compute WebAuthn digest from BCS-encoded WebAuthn envelope data
    fun compute_webauthn_digest_from_bcs(webauthn_envelope_bytes: &vector<u8>, tx_hash: &vector<u8>): vector<u8> {
        let webauthn_envelope = bcs::from_bytes<WebauthnEnvelopeData>(*webauthn_envelope_bytes);
        let WebauthnEnvelopeData {
            authenticator_data,
            client_data_json,
        } = webauthn_envelope;
        
        // Verify that the challenge in client_data_json matches tx_hash
        let client_data = json::from_json<ClientData>(client_data_json);
        let challenge = client_data.challenge;
        let tx_hash_in_client_data = base64::decode(string::bytes(&challenge));
        assert!(tx_hash_in_client_data == *tx_hash, auth_validator::error_validate_invalid_authenticator());
        
        // Reconstruct WebAuthn message: authenticator_data || SHA256(client_data_json)
        let cd_hash = hash::sha2_256(client_data_json);
        let msg = authenticator_data;
        vector::append(&mut msg, cd_hash);
        
        msg
    }

    /// Main validation function
    public(friend) fun validate(authenticator_payload: vector<u8>): DID {
        // 1. Parse authenticator payload
        let auth_payload = parse_did_auth_payload(&authenticator_payload);
        
        // 2. Derive DID from sender address
        let sender = tx_context::sender();
        let sender_did = did::new_rooch_did_by_address(sender);
        let _did_identifier = did::get_did_identifier_string(&sender_did);
        
        // 3. Get DID Document
        let did_doc = did::get_did_document_by_address(sender);
        
        // 4. Verify the verification method is authorized for authentication
        assert!(
            vector::contains(
                did::doc_authentication_methods(did_doc), 
                &auth_payload.vm_fragment
            ),
            auth_validator::error_validate_invalid_authenticator()
        );
        
        // 5. Get verification method details
        let vm_opt = did::doc_verification_method(
            did_doc, 
            &auth_payload.vm_fragment
        );
        assert!(
            option::is_some(&vm_opt),
            auth_validator::error_validate_invalid_authenticator()
        );
        let vm = option::extract(&mut vm_opt);
        
        // 6. Compute message digest based on envelope type
        let tx_hash = tx_context::tx_hash();
        let digest = compute_digest(
            &tx_hash, 
            auth_payload.envelope, 
            &auth_payload.message
        );
        
        // 7. Verify signature using DID's signature verification
        let valid = did::verify_signature_by_type(
            digest,
            auth_payload.signature,
            did::verification_method_public_key_multibase(&vm),
            did::verification_method_type(&vm)
        );
        
        assert!(valid, auth_validator::error_validate_invalid_authenticator());
        
        // Return the DID for transaction context
        sender_did
    }
}
