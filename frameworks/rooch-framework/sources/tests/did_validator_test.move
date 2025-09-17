// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::did_validator_test {
    use std::vector;
    use rooch_framework::genesis;
    use rooch_framework::did_validator;
    use rooch_framework::session_key;

    #[test]
    fun test_did_auth_validator_id() {
        assert!(did_validator::auth_validator_id() == 4, 1);
    }

    #[test]
    #[expected_failure(abort_code = 1010)]
    fun test_did_auth_empty_payload() {
        genesis::init_for_test();
        
        let payload = vector::empty<u8>();
        did_validator::validate(payload);
    }

    #[test]
    #[expected_failure(abort_code = 1010)]
    fun test_did_auth_short_payload() {
        genesis::init_for_test();
        
        let payload = vector::empty<u8>();
        vector::push_back(&mut payload, session_key::signature_scheme_ed25519());
        // Missing other required fields
        
        did_validator::validate(payload);
    }

    #[test]
    #[expected_failure(abort_code = 1010)]
    fun test_did_auth_invalid_scheme() {
        genesis::init_for_test();
        
        // Create payload with invalid scheme
        let payload = vector::empty<u8>();
        vector::push_back(&mut payload, 99); // Invalid scheme
        vector::push_back(&mut payload, 1);  // Fragment length
        vector::push_back(&mut payload, 65); // Fragment 'A'
        
        // Add dummy signature
        let i = 0;
        while (i < 64) {
            vector::push_back(&mut payload, (i as u8));
            i = i + 1;
        };

        did_validator::validate(payload);
    }
}
