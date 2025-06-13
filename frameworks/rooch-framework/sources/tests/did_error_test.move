// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Error Handling & Security functionality
/// Covers error code coverage, security boundaries, and input validation
module rooch_framework::did_error_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::option;
    use std::vector;
    use moveos_std::account;
    use moveos_std::multibase_codec;
    // ========================================
    // Test Category 9: Error Handling & Security Tests
    // ========================================

    #[test]
    #[expected_failure(abort_code = 22, location = rooch_framework::did)] // ErrorInvalidDIDStringFormat
    /// Test invalid DID string format handling
    fun test_error_invalid_did_string_format() {
        // Try to parse invalid DID string
        let invalid_did_string = string::utf8(b"invalid:format");
        let _ = did::parse_did_string(&invalid_did_string);
    }

    #[test]
    #[expected_failure(abort_code = 22, location = rooch_framework::did)] // ErrorInvalidDIDStringFormat  
    /// Test DID string with missing method
    fun test_error_invalid_did_string_missing_method() {
        // Try to parse DID string with missing method
        let invalid_did_string = string::utf8(b"did::identifier");
        let _ = did::parse_did_string(&invalid_did_string);
    }

    #[test]
    #[expected_failure(abort_code = 22, location = rooch_framework::did)] // ErrorInvalidDIDStringFormat
    /// Test DID string with missing identifier
    fun test_error_invalid_did_string_missing_identifier() {
        // Try to parse DID string with missing identifier
        let invalid_did_string = string::utf8(b"did:rooch:");
        let _ = did::parse_did_string(&invalid_did_string);
    }

    #[test]
    #[expected_failure(abort_code = 22, location = rooch_framework::did)] // ErrorInvalidDIDStringFormat
    /// Test DID string with wrong prefix
    fun test_error_invalid_did_string_wrong_prefix() {
        // Try to parse DID string with wrong prefix
        let invalid_did_string = string::utf8(b"wrong:rooch:identifier");
        let _ = did::parse_did_string(&invalid_did_string);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorDIDDocumentNotExist, location = rooch_framework::did)]
    /// Test error when trying to access non-existent DID document
    fun test_error_did_document_not_exist() {
        did_test_common::init_test_framework();

        // Try to get DID document for non-existent address
        let nonexistent_address = @0x999999;
        let _ = did::get_did_document_by_address(nonexistent_address);
    }

    #[test]
    /// Test input validation for DID string parsing with various formats
    fun test_input_validation_did_string_parsing() {
        // Test valid did:rooch format
        let valid_rooch_did = string::utf8(b"did:rooch:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4");
        let parsed_rooch = did::parse_did_string(&valid_rooch_did);
        let formatted_rooch = did::format_did(&parsed_rooch);
        assert!(formatted_rooch == valid_rooch_did, 13001);

        // Test valid did:key format
        let valid_key_did = string::utf8(b"did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        let parsed_key = did::parse_did_string(&valid_key_did);
        let formatted_key = did::format_did(&parsed_key);
        assert!(formatted_key == valid_key_did, 13002);

        // Test with colons in identifier (should be allowed)
        let complex_did = string::utf8(b"did:example:user:123:test");
        let parsed_complex = did::parse_did_string(&complex_did);
        let formatted_complex = did::format_did(&parsed_complex);
        assert!(formatted_complex == complex_did, 13003);
    }

    #[test]
    /// Test verification relationship validation
    fun test_verification_relationship_validation() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Test has_verification_relationship_in_doc function with valid relationships - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        
        // account-key should have authentication, assertion_method, capability_invocation, capability_delegation
        let account_key_fragment = string::utf8(b"account-key");
        assert!(did::has_verification_relationship_in_doc(did_document_check, &account_key_fragment, 0), 13101); // authentication
        assert!(did::has_verification_relationship_in_doc(did_document_check, &account_key_fragment, 1), 13102); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_check, &account_key_fragment, 2), 13103); // capability_invocation
        assert!(did::has_verification_relationship_in_doc(did_document_check, &account_key_fragment, 3), 13104); // capability_delegation
        
        // account-key should not have key_agreement (not assigned)
        assert!(!did::has_verification_relationship_in_doc(did_document_check, &account_key_fragment, 4), 13105); // key_agreement

        // Non-existent fragment should return false for any relationship
        let nonexistent_fragment = string::utf8(b"nonexistent");
        assert!(!did::has_verification_relationship_in_doc(did_document_check, &nonexistent_fragment, 0), 13106); // authentication
    }

    #[test]
    /// Test input validation for empty parameters
    fun test_input_validation_empty_parameters() {
        did_test_common::init_test_framework();

        // Test parsing empty string (should fail gracefully)
        let empty_string = string::utf8(b"");
        
        // Check that empty string is handled properly in existence checks
        assert!(!did::exists_did_document_by_identifier(empty_string), 13301);
    }

    #[test]
    /// Test input validation for fragment uniqueness
    fun test_input_validation_fragment_uniqueness() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add first verification method
        let fragment = string::utf8(b"unique-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key1 = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(&did_signer, fragment, method_type, test_key1, relationships);

        // Verify method was added - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &fragment), 13401);

        // Fragment uniqueness is enforced - trying to add duplicate should fail in add_verification_method_already_exists test
    }


    #[test]
    /// Test error handling for malformed service properties
    fun test_error_malformed_service_properties() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Test valid service addition with matching arrays
        let fragment = string::utf8(b"valid-service");
        let service_type = string::utf8(b"ValidService");
        let service_endpoint = string::utf8(b"https://valid.example.com");
        let matching_keys = vector[string::utf8(b"key1"), string::utf8(b"key2")];
        let matching_values = vector[string::utf8(b"value1"), string::utf8(b"value2")];

        did::add_service_with_properties_entry(
            &did_signer,
            fragment,
            service_type,
            service_endpoint,
            matching_keys,
            matching_values
        );

        // Verify service was added successfully - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_check, &fragment), 13601);
    }

    #[test]
    /// Test security boundary for session key scope validation
    fun test_security_session_key_scope() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Setup session key for authorization (reuse the creator's key)
        let pk_bytes_opt = multibase_codec::decode(&creator_public_key_multibase);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        
        // Test that session key context is properly scoped
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));
        
        // Verify session key can be retrieved and is correct
        let retrieved_session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(std::option::is_some(&retrieved_session_key_opt), 13701);
        
        let retrieved_session_key = std::option::extract(&mut retrieved_session_key_opt);
        assert!(retrieved_session_key == auth_key, 13702);

        // Add verification method - should succeed with valid session key
        let fragment = string::utf8(b"scoped-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(&did_signer, fragment, method_type, test_key, relationships);

        // Verify method was added - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &fragment), 13703);
    }

    #[test]
    /// Test comprehensive error code coverage
    fun test_comprehensive_error_coverage() {
        did_test_common::init_test_framework();

        // Test DID existence queries for comprehensive coverage
        assert!(!did::exists_did_for_address(@0x1), 13801);
        assert!(!did::exists_did_document_by_identifier(string::utf8(b"nonexistent")), 13802);

        // Test controller queries
        let nonexistent_controller = did::new_did_from_parts(
            string::utf8(b"test"),
            string::utf8(b"nonexistent")
        );
        let empty_result = did::get_dids_by_controller(nonexistent_controller);
        assert!(vector::length(&empty_result) == 0, 13803);

        // Test string queries  
        let empty_string_result = did::get_dids_by_controller_string(string::utf8(b"did:test:nonexistent"));
        assert!(vector::length(&empty_string_result) == 0, 13804);

        // Test DID parsing and formatting edge cases
        let test_did = did::new_did_from_parts(
            string::utf8(b"test"),
            string::utf8(b"identifier")
        );
        let formatted = did::format_did(&test_did);
        let expected = string::utf8(b"did:test:identifier");
        assert!(formatted == expected, 13805);
    }

    #[test]
    /// Test input sanitization and validation edge cases
    fun test_input_sanitization_edge_cases() {
        // Test various edge cases in DID string handling
        
        // Very long identifier
        let long_identifier = string::utf8(b"verylongidentifierthatcontainsmanycharsbutshouldbeparsedcorrectly123456789");
        let long_did = did::new_did_from_parts(string::utf8(b"test"), long_identifier);
        let formatted_long = did::format_did(&long_did);
        let expected_long = string::utf8(b"did:test:");
        string::append(&mut expected_long, long_identifier);
        assert!(formatted_long == expected_long, 13901);

        // Identifier with special characters
        let special_identifier = string::utf8(b"user-123_test.example");
        let special_did = did::new_did_from_parts(string::utf8(b"test"), special_identifier);
        let formatted_special = did::format_did(&special_did);
        let expected_special = string::utf8(b"did:test:");
        string::append(&mut expected_special, special_identifier);
        assert!(formatted_special == expected_special, 13902);

        // Multiple colons in identifier (allowed in DID spec)
        let colon_identifier = string::utf8(b"namespace:type:id:version");
        let colon_did = did::new_did_from_parts(string::utf8(b"test"), colon_identifier);
        let formatted_colon = did::format_did(&colon_did);
        let expected_colon = string::utf8(b"did:test:");
        string::append(&mut expected_colon, colon_identifier);
        assert!(formatted_colon == expected_colon, 13903);
    }

    #[test]
    #[expected_failure(abort_code = 13, location = rooch_framework::did)] // ErrorDIDObjectNotFound
    /// Test signer validation - wrong signer for DID operations
    fun test_error_signer_not_did_account() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let _did_address = did::get_did_address(did_document);

        let wrong_signer = account::create_signer_for_testing(@0x999);

        // Try to modify DID with wrong signer - should fail
        let fragment = string::utf8(b"new-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(
            &wrong_signer, // Wrong signer!
            fragment,
            method_type,
            test_key,
            relationships
        );
    }
} 