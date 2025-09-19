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
    use rooch_framework::did::{Self, DID};
    use rooch_framework::session_key;
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

    /// Error codes for DID validator (using 101xxx range to avoid conflicts)
    /// DID validator specific errors: 101001-101999
    
    /// Invalid BCS deserialization of DID auth payload
    const ErrorInvalidDIDAuthPayload: u64 = 101001;
    /// Invalid envelope type in DID auth payload
    const ErrorInvalidEnvelopeType: u64 = 101002;
    /// DID document not found for sender address
    const ErrorDIDDocumentNotFound: u64 = 101003;
    /// Verification method not authorized for authentication
    const ErrorVerificationMethodNotAuthorized: u64 = 101004;
    /// Verification method not found in DID document
    const ErrorVerificationMethodNotFound: u64 = 101005;
    /// Invalid message for envelope type
    const ErrorInvalidEnvelopeMessage: u64 = 101006;
    /// Signature verification failed
    const ErrorSignatureVerificationFailed: u64 = 101007;

    struct DIDValidator has store, drop {}

    #[data_struct]
    struct DIDAuthPayload has copy, store, drop {
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

    /// Parse DID authenticator payload using BCS deserialization
    fun parse_did_auth_payload(authenticator_payload: &vector<u8>): DIDAuthPayload {
        // Use BCS to deserialize the payload
        let auth_payload = bcs::from_bytes<DIDAuthPayload>(*authenticator_payload);
        
        // Validate envelope type
        assert!(
            auth_payload.envelope == ENVELOPE_RAW_TX_HASH || 
            auth_payload.envelope == ENVELOPE_BITCOIN_MESSAGE_V0 || 
            auth_payload.envelope == ENVELOPE_WEBAUTHN_V0,
            ErrorInvalidEnvelopeType
        );
        
        auth_payload
    }


    /// Compute digest based on envelope type (reuse session validator logic)
    fun compute_digest(tx_hash: &vector<u8>, envelope: u8, message_option: &Option<vector<u8>>): vector<u8> {
        if (envelope == ENVELOPE_RAW_TX_HASH) {
            // RawTxHash: digest = tx_hash
            *tx_hash
        } else if (envelope == ENVELOPE_BITCOIN_MESSAGE_V0) {
            // BitcoinMessageV0: verify message matches canonical template, then compute Bitcoin digest
            assert!(option::is_some(message_option), ErrorInvalidEnvelopeMessage);
            let message = option::borrow(message_option);
            
            // Verify message matches canonical template
            let expected_template = session_key::build_canonical_template(tx_hash);
            assert!(*message == expected_template, ErrorInvalidEnvelopeMessage);
            
            // Compute Bitcoin message digest
            session_key::bitcoin_message_digest(message)
        } else if (envelope == ENVELOPE_WEBAUTHN_V0) {
            // WebAuthn: reconstruct message as authenticator_data || SHA256(client_data_json)
            assert!(option::is_some(message_option), ErrorInvalidEnvelopeMessage);
            let webauthn_payload_bytes = option::borrow(message_option);
            
            // Compute WebAuthn digest (same logic as session_validator)
            compute_webauthn_digest_from_bcs(webauthn_payload_bytes, tx_hash)
        } else {
            // Unknown envelope
            abort ErrorInvalidEnvelopeType
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
        assert!(tx_hash_in_client_data == *tx_hash, ErrorInvalidEnvelopeMessage);
        
        // Reconstruct WebAuthn message: authenticator_data || SHA256(client_data_json)
        let cd_hash = hash::sha2_256(client_data_json);
        let msg = authenticator_data;
        vector::append(&mut msg, cd_hash);
        
        msg
    }


    /// Main validation function
    public(friend) fun validate(authenticator_payload: vector<u8>): (DID, String) {
        // 1. Parse authenticator payload
        let auth_payload = parse_did_auth_payload(&authenticator_payload);
        
        // 2. Derive DID from sender address
        let sender = tx_context::sender();
        let sender_did = did::new_rooch_did_by_address(sender);
        
        // 3. Get DID Document (this will abort if DID document doesn't exist)
        let did_doc = did::get_did_document_by_address(sender);
        
        // 4. Verify the verification method is authorized for authentication
        assert!(
            vector::contains(
                did::doc_authentication_methods(did_doc), 
                &auth_payload.vm_fragment
            ),
            ErrorVerificationMethodNotAuthorized
        );
        
        // 5. Get verification method details
        let vm_opt = did::doc_verification_method(
            did_doc, 
            &auth_payload.vm_fragment
        );
        assert!(
            option::is_some(&vm_opt),
            ErrorVerificationMethodNotFound
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
        
        assert!(valid, ErrorSignatureVerificationFailed);
        
        // Return the DID and vm_fragment for transaction context
        (sender_did, auth_payload.vm_fragment)
    }
}
