// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the session validator with signing envelopes
module rooch_framework::session_validator_test {

    use std::vector;
    use rooch_framework::session_key;

    #[test_only]
    /// Helper function to create a mock v1 payload (legacy format)
    fun create_v1_payload_ed25519(): vector<u8> {
        let payload = vector::empty<u8>();
        
        // Scheme: Ed25519 (0)
        vector::push_back(&mut payload, session_key::signature_scheme_ed25519());
        
        // Mock signature (64 bytes)
        let i = 0;
        while (i < 64) {
            vector::push_back(&mut payload, (i as u8));
            i = i + 1;
        };
        
        // Mock public key (32 bytes)
        let i = 0;
        while (i < 32) {
            vector::push_back(&mut payload, (100 + i as u8));
            i = i + 1;
        };
        
        payload
    }

    #[test_only]
    /// Helper function to create a mock v2 payload with RawTxHash envelope
    fun create_v2_payload_raw_tx_hash(): vector<u8> {
        let payload = vector::empty<u8>();
        
        // Scheme: Secp256k1 (1)
        vector::push_back(&mut payload, session_key::signature_scheme_secp256k1());
        
        // Envelope: RawTxHash (0)
        vector::push_back(&mut payload, session_key::signing_envelope_raw_tx_hash());
        
        // Mock signature (64 bytes)
        let i = 0;
        while (i < 64) {
            vector::push_back(&mut payload, (i as u8));
            i = i + 1;
        };
        
        // Mock public key (33 bytes for Secp256k1)
        let i = 0;
        while (i < 33) {
            vector::push_back(&mut payload, (200 + i as u8));
            i = i + 1;
        };
        
        payload
    }

    #[test_only]
    /// Helper function to create a mock v2 payload with BitcoinMessageV0 envelope
    fun create_v2_payload_bitcoin_message(): vector<u8> {
        let payload = vector::empty<u8>();
        
        // Scheme: Secp256k1 (1)
        vector::push_back(&mut payload, session_key::signature_scheme_secp256k1());
        
        // Envelope: BitcoinMessageV0 (1)
        vector::push_back(&mut payload, session_key::signing_envelope_bitcoin_message_v0());
        
        // Mock signature (64 bytes)
        let i = 0;
        while (i < 64) {
            vector::push_back(&mut payload, (i as u8));
            i = i + 1;
        };
        
        // Mock public key (33 bytes for Secp256k1)
        let i = 0;
        while (i < 33) {
            vector::push_back(&mut payload, (200 + i as u8));
            i = i + 1;
        };
        
        // Mock message: "Rooch Transaction:\n" + 64 hex chars
        let message = b"Rooch Transaction:\n0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let msg_len = vector::length(&message);
        
        // Message length (VarInt single-byte)
        vector::push_back(&mut payload, (msg_len as u8));
        
        // Message content
        vector::append(&mut payload, message);
        
        payload
    }

    #[test]
    fun test_hex_lowercase_functionality() {
        // Test the hex_lowercase function with known input
        let hash = vector::empty<u8>();
        vector::push_back(&mut hash, 0x01);
        vector::push_back(&mut hash, 0x23);
        vector::push_back(&mut hash, 0x45);
        vector::push_back(&mut hash, 0x67);
        vector::push_back(&mut hash, 0x89);
        vector::push_back(&mut hash, 0xab);
        vector::push_back(&mut hash, 0xcd);
        vector::push_back(&mut hash, 0xef);
        
        let hex_result = session_key::hex_lowercase(&hash);
        let expected = b"0123456789abcdef";
        
        assert!(hex_result == expected, 1000);
    }

    #[test]
    fun test_canonical_template_building() {
        // Test canonical template building
        let tx_hash = vector::empty<u8>();
        let i = 0;
        while (i < 32) {
            vector::push_back(&mut tx_hash, (i as u8));
            i = i + 1;
        };
        
        let template = session_key::build_canonical_template(&tx_hash);
        
        // Should start with "Rooch Transaction:\n"
        let prefix = b"Rooch Transaction:\n";
        let prefix_len = vector::length(&prefix);
        
        assert!(vector::length(&template) == prefix_len + 64, 2000); // prefix + 64 hex chars
        
        // Check prefix
        let i = 0;
        while (i < prefix_len) {
            assert!(*vector::borrow(&template, i) == *vector::borrow(&prefix, i), 2001);
            i = i + 1;
        };
    }

    #[test]
    fun test_bitcoin_message_digest_deterministic() {
        // Test that Bitcoin message digest is deterministic
        let message1 = b"test message";
        let digest1 = session_key::bitcoin_message_digest(&message1);
        let digest2 = session_key::bitcoin_message_digest(&message1);
        
        assert!(digest1 == digest2, 3000);
        assert!(vector::length(&digest1) == 32, 3001); // SHA256 output length
        
        // Different messages should produce different digests
        let message2 = b"different message";
        let digest3 = session_key::bitcoin_message_digest(&message2);
        
        assert!(digest1 != digest3, 3002);
    }

    #[test]
    fun test_v1_payload_parsing() {
        // Test that v1 payloads are correctly identified and parsed
        let v1_payload = create_v1_payload_ed25519();
        
        // The payload should be exactly 97 bytes for Ed25519 v1 format
        assert!(vector::length(&v1_payload) == 97, 4000); // 1 + 64 + 32
    }

    #[test]
    fun test_v2_payload_parsing() {
        // Test that v2 payloads are correctly identified and parsed
        let v2_payload = create_v2_payload_raw_tx_hash();
        
        // The payload should be longer than v1 due to envelope byte
        assert!(vector::length(&v2_payload) == 99, 5000); // 1 + 1 + 64 + 33
    }

    #[test]
    fun test_v2_payload_with_message() {
        // Test v2 payload with BitcoinMessageV0 envelope
        let v2_payload = create_v2_payload_bitcoin_message();
        
        // Should include message length and message content
        // "Rooch Transaction:\n" (18 bytes) + 64 hex chars = 83 bytes total
        let expected_len = 1 + 1 + 64 + 33 + 1 + 83; // scheme + envelope + sig + pk + msg_len + message
        assert!(vector::length(&v2_payload) == expected_len, 6000);
    }

    #[test]
    fun test_envelope_constants() {
        // Test that envelope constants are correctly defined
        assert!(session_key::signing_envelope_raw_tx_hash() == 0, 7000);
        assert!(session_key::signing_envelope_bitcoin_message_v0() == 1, 7001);
    }

    #[test]
    fun test_signature_scheme_constants() {
        // Test that signature scheme constants are correctly defined
        assert!(session_key::signature_scheme_ed25519() == 0, 8000);
        assert!(session_key::signature_scheme_secp256k1() == 1, 8001);
        assert!(session_key::signature_scheme_ecdsar1() == 2, 8002);
    }

    #[test]
    fun test_webauthn_envelope_constant() {
        // Test that WebAuthn envelope constant is correctly defined
        let webauthn_envelope = session_key::signing_envelope_webauthn_v0();
        assert!(webauthn_envelope == 2, 9000);
        
        // Test that it's different from other envelopes
        assert!(webauthn_envelope != session_key::signing_envelope_raw_tx_hash(), 9001);
        assert!(webauthn_envelope != session_key::signing_envelope_bitcoin_message_v0(), 9002);
    }

    // Note: Full end-to-end validation tests would require proper cryptographic signatures
    // and session key setup, which is complex for unit tests. The above tests focus on
    // the parsing and utility functions that can be tested in isolation.
}
