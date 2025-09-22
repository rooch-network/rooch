// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::auth_validator_test {
    use std::option;
    use std::string;
    use moveos_std::tx_context;
    use rooch_framework::auth_validator;
    use rooch_framework::bitcoin_address;

    #[test]
    fun test_new_tx_validate_result_with_session_key() {
        let session_key = b"test_session_key";
        let bitcoin_addr = bitcoin_address::empty();
        
        let result = auth_validator::new_tx_validate_result_with_optional_data(
            1,
            option::none(),
            option::some(session_key),
            option::none(),
            bitcoin_addr
        );
        
        // Set the result in context for testing
        rooch_framework::genesis::init_for_test();
        tx_context::set_attribute_for_testing(result);
        
        // Test that we can get the session key back
        let session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(option::is_some(&session_key_opt), 1);
        assert!(option::extract(&mut session_key_opt) == session_key, 2);
        
        // Test that DID VM fragment is None
        let did_vm_fragment_opt = auth_validator::get_did_vm_fragment_from_ctx_option();
        assert!(option::is_none(&did_vm_fragment_opt), 3);
    }

    #[test]
    fun test_new_tx_validate_result_with_did_vm_fragment() {
        let vm_fragment = string::utf8(b"test_vm_fragment");
        let bitcoin_addr = bitcoin_address::empty();
        
        let result = auth_validator::new_tx_validate_result_with_optional_data(
            4, // DID validator ID
            option::none(),
            option::none(),
            option::some(vm_fragment),
            bitcoin_addr
        );
        
        // Set the result in context for testing
        rooch_framework::genesis::init_for_test();
        tx_context::set_attribute_for_testing(result);
        
        // Test that session key is None (since it's encoded DID VM fragment)
        let session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(option::is_none(&session_key_opt), 1);
        
        // Test that we can get the DID VM fragment back
        let did_vm_fragment_opt = auth_validator::get_did_vm_fragment_from_ctx_option();
        assert!(option::is_some(&did_vm_fragment_opt), 2);
        let extracted_fragment = option::extract(&mut did_vm_fragment_opt);
        assert!(extracted_fragment == vm_fragment, 3);
    }

    #[test]
    fun test_vm_fragment_priority_over_session_key() {
        let session_key = b"test_session_key";
        let vm_fragment = string::utf8(b"test_vm_fragment");
        let bitcoin_addr = bitcoin_address::empty();
        
        // When both are provided, vm_fragment should take priority
        let result = auth_validator::new_tx_validate_result_with_optional_data(
            4, // DID validator ID
            option::none(),
            option::some(session_key),
            option::some(vm_fragment),
            bitcoin_addr
        );
        
        // Set the result in context for testing
        rooch_framework::genesis::init_for_test();
        tx_context::set_attribute_for_testing(result);
        
        // Test that session key is None (since vm_fragment takes priority)
        let session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(option::is_none(&session_key_opt), 1);
        
        // Test that we can get the DID VM fragment back
        let did_vm_fragment_opt = auth_validator::get_did_vm_fragment_from_ctx_option();
        assert!(option::is_some(&did_vm_fragment_opt), 2);
        let extracted_fragment = option::extract(&mut did_vm_fragment_opt);
        assert!(extracted_fragment == vm_fragment, 3);
    }
}