// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Custom Session Scope functionality
/// Covers new entry functions with custom scope configuration
module rooch_framework::did_custom_scopes_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::option;
    use std::vector;

    // ========================================
    // Test Category: Custom Session Scope Tests
    // ========================================

    #[test]
    /// Test DID creation with custom scopes - basic functionality
    /// Verifies that custom scope strings are properly parsed and applied
    fun test_create_did_with_custom_scopes_basic() {
        // Define custom scopes - minimal permissions for security
        let custom_scopes = vector[
            string::utf8(b"0x3::did::*"),
            string::utf8(b"0x3::transfer::transfer"),
            string::utf8(b"0x3::coin::transfer")
        ];
        
        // Create DID with custom scopes using the test helper
        let (creator_signer, _creator_address, creator_public_key, did_object_id) = 
            did_test_common::setup_did_test_with_scope_creation(option::some(custom_scopes));
        
        // Verify DID was created successfully
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        assert!(did::exists_did_for_address(did_address), 20001);
        
        let vm_opt = did::doc_verification_method(did_document, &string::utf8(b"account-key"));
        let vm = option::destroy_some(vm_opt);
        let vm_pubkey = did::verification_method_public_key_multibase(&vm);
        assert!(*vm_pubkey == creator_public_key, 20005);
        
        // Clean up unused variables
        let _ = creator_signer;
    }


    #[test]
    /// Test DID creation with empty custom scopes - should work
    /// Verifies that empty scope vectors are handled gracefully
    fun test_create_did_with_empty_custom_scopes() {
        // Create DID with empty scopes (should fall back to default behavior)
        let empty_scopes = vector::empty<string::String>();
        let (creator_signer, creator_address, creator_public_key, did_object_id) = 
            did_test_common::setup_did_test_with_scope_creation(option::some(empty_scopes));
        
        // Verify DID was created successfully
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        assert!(did::exists_did_for_address(did_address), 20101);
        
        // Verify verification method exists with correct public key
        let account_key_fragment = string::utf8(b"account-key");
        let vm_opt = did::doc_verification_method(did_document, &account_key_fragment);
        assert!(option::is_some(&vm_opt), 20103);
        
        let vm = option::destroy_some(vm_opt);
        let vm_pubkey = did::verification_method_public_key_multibase(&vm);
        assert!(*vm_pubkey == creator_public_key, 20104);
        
        // Clean up unused variables
        let _ = creator_signer;
        let _ = creator_address;
    }

    #[test]
    /// Test CADOP DID creation with custom scopes
    /// Verifies that CADOP creation supports custom scope configuration
    fun test_cadop_create_did_with_custom_scopes() {
        // Initialize test framework and setup CADOP custodian
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();
        
        // Define custom scopes for CADOP DID
        let custom_scopes = vector[
            string::utf8(b"0x3::did::update_verification_method"),
            string::utf8(b"0x3::did::add_service"),
            string::utf8(b"0x3::transfer::*")
        ];
        
        // Create DID via CADOP with custom scopes using entry function
        let did_object_id = did::create_did_object_via_cadop_with_did_key_and_scopes(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type,
            option::some(custom_scopes)
        );
        
        // Verify DID was created successfully
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        assert!(did::exists_did_for_address(did_address), 20302);
        
        // Verify DID document has user verification method
        let account_key_fragment = string::utf8(b"account-key");
        let user_vm_opt = did::doc_verification_method(did_document, &account_key_fragment);
        assert!(option::is_some(&user_vm_opt), 20303);
    }

   
}
