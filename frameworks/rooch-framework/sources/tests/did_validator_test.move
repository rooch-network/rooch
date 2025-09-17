// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::did_validator_test {
    use std::vector;
    use rooch_framework::genesis;
    use rooch_framework::did_validator;
    use rooch_framework::session_key;
    
    // Error code constants for cleaner tests
    const BCS_ERROR_INVALID_BYTES: u64 = 2;
    const DID_ERROR_INVALID_ENVELOPE_TYPE: u64 = 101002;

    #[test]
    fun test_did_auth_validator_id() {
        assert!(did_validator::auth_validator_id() == 4, 1);
    }

    #[test]
    #[expected_failure(abort_code = BCS_ERROR_INVALID_BYTES, location = moveos_std::bcs)]
    fun test_did_auth_empty_payload() {
        genesis::init_for_test();
        
        let payload = vector::empty<u8>();
        did_validator::validate(payload);
    }

    #[test]
    #[expected_failure(abort_code = BCS_ERROR_INVALID_BYTES, location = moveos_std::bcs)]
    fun test_did_auth_short_payload() {
        genesis::init_for_test();
        
        let payload = vector::empty<u8>();
        vector::push_back(&mut payload, session_key::signature_scheme_ed25519());
        // Missing other required fields - will fail BCS deserialization
        
        did_validator::validate(payload);
    }

    #[test]
    #[expected_failure(abort_code = DID_ERROR_INVALID_ENVELOPE_TYPE, location = rooch_framework::did_validator)]
    fun test_did_auth_invalid_envelope() {
        genesis::init_for_test();
        
        // Create a valid BCS-encoded DIDAuthPayload with invalid envelope type
        // BCS format: scheme(u8) + envelope(u8) + vm_fragment(string) + signature(vector<u8>) + message(Option<vector<u8>>)
        let mut_payload = vector::empty<u8>();
        
        // scheme: u8 = 0 (Ed25519)
        vector::push_back(&mut mut_payload, 0);
        
        // envelope: u8 = 99 (invalid envelope type)
        vector::push_back(&mut mut_payload, 99);
        
        // vm_fragment: string = "key-1" (length-prefixed)
        vector::push_back(&mut mut_payload, 5); // string length
        vector::append(&mut mut_payload, b"key-1");
        
        // signature: vector<u8> = empty (length-prefixed)
        vector::push_back(&mut mut_payload, 0); // vector length
        
        // message: Option<vector<u8>> = None
        vector::push_back(&mut mut_payload, 0); // option tag for None
        
        let payload = mut_payload;
        did_validator::validate(payload);
    }

    #[test]
    #[expected_failure] // Will fail when trying to get DID document, but passes BCS and envelope validation
    fun test_valid_bcs_and_envelope_but_no_did() {
        // Test that valid BCS format and envelope pass initial validation
        // but fail later when DID document is not found
        genesis::init_for_test();
        
        // Create a valid BCS-encoded DIDAuthPayload with valid envelope type
        let mut_payload = vector::empty<u8>();
        
        // scheme: u8 = 0 (Ed25519)
        vector::push_back(&mut mut_payload, 0);
        
        // envelope: u8 = 0 (RawTxHash - valid envelope)
        vector::push_back(&mut mut_payload, 0);
        
        // vm_fragment: string = "key-1" (length-prefixed)
        vector::push_back(&mut mut_payload, 5); // string length
        vector::append(&mut mut_payload, b"key-1");
        
        // signature: vector<u8> = empty (length-prefixed)
        vector::push_back(&mut mut_payload, 0); // vector length
        
        // message: Option<vector<u8>> = None
        vector::push_back(&mut mut_payload, 0); // option tag for None
        
        let payload = mut_payload;
        
        // This should pass BCS parsing and envelope validation
        // but fail when trying to get DID document (since we don't have a real DID setup)
        did_validator::validate(payload);
    }
}
