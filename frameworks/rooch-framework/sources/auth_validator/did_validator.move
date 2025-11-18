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
    use moveos_std::hash;
    use moveos_std::json;
    use moveos_std::bcs;
    use moveos_std::base64;
    use moveos_std::hex;
    use moveos_std::consensus_codec;

    friend rooch_framework::transaction_validator;
    
    #[test_only]
    friend rooch_framework::did_validator_test;

    /// DID auth validator ID
    const DID_VALIDATOR_ID: u64 = 4;

    /// Envelope types (same as session validator)
    const ENVELOPE_RAW_TX_HASH: u8 = 0x00;
    const ENVELOPE_BITCOIN_MESSAGE_V0: u8 = 0x01;
    const ENVELOPE_WEBAUTHN_V0: u8 = 0x02;

    const BITCOIN_MESSAGE_PREFIX: vector<u8> = b"Bitcoin Signed Message:\n";
    const ROOCH_TRANSACTION_MESSAGE_PREFIX: vector<u8> = b"Rooch Transaction:\n";

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


    /// Build Rooch transaction message for Bitcoin signature verification
    /// Uses the same format as auth_payload.move: "Rooch Transaction:\n" + hex(tx_hash)
    /// This message format is used in BitcoinMessageV0 envelope for DID authentication
    public fun build_rooch_transaction_message(tx_hash: vector<u8>): vector<u8> {
        let prefix = ROOCH_TRANSACTION_MESSAGE_PREFIX;
        let hex_hash = hex::encode(tx_hash);
        
        let message = vector::empty<u8>();
        vector::append(&mut message, prefix);
        vector::append(&mut message, hex_hash);
        
        message
    }

    /// Encode Bitcoin message using the same format as TypeScript BitcoinSignMessage
    /// Format: \u0018 + "Bitcoin Signed Message:\n" + varint(message_len) + message
    public fun encode_bitcoin_message(message: vector<u8>): vector<u8> {
        let encoder = consensus_codec::encoder();  
        
        // Add Bitcoin prefix
        let bitcoin_prefix = BITCOIN_MESSAGE_PREFIX;
        consensus_codec::emit_var_slice(&mut encoder, bitcoin_prefix);
        
        consensus_codec::emit_var_slice(&mut encoder, message);
        
        consensus_codec::unpack_encoder(encoder)
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
    fun compute_digest(tx_hash: vector<u8>, envelope: u8, message_option: Option<vector<u8>>): vector<u8> {
        if (envelope == ENVELOPE_RAW_TX_HASH) {
            // RawTxHash: digest = tx_hash
            tx_hash
        } else if (envelope == ENVELOPE_BITCOIN_MESSAGE_V0) {
            // BitcoinMessageV0: use the same logic as bitcoin_validator
            assert!(option::is_some(&message_option), ErrorInvalidEnvelopeMessage);
            let message = option::destroy_some(message_option);
            
            // Verify message matches expected Rooch transaction message format
            let expected_message = build_rooch_transaction_message(tx_hash);
            assert!(message == expected_message, ErrorInvalidEnvelopeMessage);
            
            // Encode Bitcoin message using the extracted method
            let full_message = encode_bitcoin_message(message);
            
            // Apply single SHA256 like bitcoin_validator (to match wallet's second hash)
            // ecdsa_k1::verify will apply another SHA256 internally (to match wallet's first hash)
            hash::sha2_256(full_message)
        } else if (envelope == ENVELOPE_WEBAUTHN_V0) {
            // WebAuthn: reconstruct message as authenticator_data || SHA256(client_data_json)
            assert!(option::is_some(&message_option), ErrorInvalidEnvelopeMessage);
            let webauthn_payload_bytes = option::destroy_some(message_option);
            
            // Compute WebAuthn digest (same logic as session_validator)
            compute_webauthn_digest_from_bcs(webauthn_payload_bytes, tx_hash)
        } else {
            // Unknown envelope
            abort ErrorInvalidEnvelopeType
        }
    }

    /// Compute WebAuthn digest from BCS-encoded WebAuthn envelope data
    fun compute_webauthn_digest_from_bcs(webauthn_envelope_bytes: vector<u8>, tx_hash: vector<u8>): vector<u8> {
        let webauthn_envelope = bcs::from_bytes<WebauthnEnvelopeData>(webauthn_envelope_bytes);
        let WebauthnEnvelopeData {
            authenticator_data,
            client_data_json,
        } = webauthn_envelope;
        
        // Verify that the challenge in client_data_json matches tx_hash
        let client_data = json::from_json<ClientData>(client_data_json);
        let challenge = client_data.challenge;
        let tx_hash_in_client_data = base64::decode(string::bytes(&challenge));
        assert!(tx_hash_in_client_data == tx_hash, ErrorInvalidEnvelopeMessage);
        
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
            tx_hash, 
            auth_payload.envelope, 
            auth_payload.message
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


    #[test_only]
    /// Create a DIDAuthPayload for testing purposes
    public fun create_test_did_auth_payload(
        envelope: u8,
        vm_fragment: String,
        signature: vector<u8>,
        message: Option<vector<u8>>
    ): vector<u8> {
        let payload = DIDAuthPayload {
            envelope,
            vm_fragment,
            signature,
            message,
        };
        bcs::to_bytes(&payload)
    }

    #[test_only]
    /// Create a WebAuthnEnvelopeData for testing purposes
    public fun create_test_webauthn_envelope(
        authenticator_data: vector<u8>,
        client_data_json: vector<u8>
    ): vector<u8> {
        let envelope = WebauthnEnvelopeData {
            authenticator_data,
            client_data_json,
        };
        bcs::to_bytes(&envelope)
    }

    #[test]
    fun test_build_rooch_transaction_message() {
        // Test with a 32-byte hash (all zeros for simplicity)
        let tx_hash = vector::empty<u8>();
        let i = 0;
        while (i < 32) {
            vector::push_back(&mut tx_hash, 0x00);
            i = i + 1;
        };
        
        let message = build_rooch_transaction_message(tx_hash);
        let expected = ROOCH_TRANSACTION_MESSAGE_PREFIX;
        vector::append(&mut expected, b"0000000000000000000000000000000000000000000000000000000000000000");
        
        assert!(message == expected, 3100);
    }

    #[test]
    fun test_encode_bitcoin_message() {
        // Test Bitcoin message encoding
        let tx_hash = x"8ba04a9fbfa161a8996db7577894f281e8e61fe4f78e6296e7821ca4c7437986";
        let rooch_transaction_message = build_rooch_transaction_message(tx_hash);
        let encoded = encode_bitcoin_message(rooch_transaction_message);
        
        // Verify the structure: should start with 0x18 (24) + "Bitcoin Signed Message:\n"
        assert!(*vector::borrow(&encoded, 0) == 24u8, 4000);
        
        // Verify Bitcoin prefix follows
        let bitcoin_prefix = BITCOIN_MESSAGE_PREFIX;
        let i = 0;
        while (i < vector::length(&bitcoin_prefix)) {
            assert!(
                *vector::borrow(&encoded, i + 1) == *vector::borrow(&bitcoin_prefix, i),
                4001 + i
            );
            i = i + 1;
        };
        
        // The encoded message should be longer than just the prefix
        assert!(vector::length(&encoded) > 25 + vector::length(&rooch_transaction_message), 4100);
    }
}
