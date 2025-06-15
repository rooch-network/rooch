// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Integration & Comprehensive Lifecycle functionality
/// Covers end-to-end scenarios, edge cases, and comprehensive system testing
module rooch_framework::did_integration_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::option;
    use std::vector;
    use moveos_std::account;
    use moveos_std::multibase_codec;

    // ========================================
    // Test Category 10: Integration & Comprehensive Tests
    // ========================================

    #[test]
    /// Test Ed25519 and Secp256k1 key types in same DID document
    fun test_mixed_key_types_in_did() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add Ed25519 verification method
        let ed25519_fragment = string::utf8(b"ed25519-key");
        let ed25519_type = string::utf8(b"Ed25519VerificationKey2020");
        let ed25519_key = did_test_common::generate_test_ed25519_multibase();
        let ed25519_relationships = vector[1u8, 4u8]; // assertion_method, key_agreement (avoid authentication)

        did::add_verification_method_entry(
            &did_signer,
            ed25519_fragment,
            ed25519_type,
            ed25519_key,
            ed25519_relationships
        );

        // Add another Secp256k1 verification method
        let secp256k1_fragment = string::utf8(b"secp256k1-key");
        let secp256k1_type = string::utf8(b"EcdsaSecp256k1VerificationKey2019");
        let secp256k1_key = did_test_common::generate_test_secp256k1_multibase(); // Different key
        let secp256k1_relationships = vector[1u8, 2u8]; // assertion_method, capability_invocation

        did::add_verification_method_entry(
            &did_signer,
            secp256k1_fragment,
            secp256k1_type,
            secp256k1_key,
            secp256k1_relationships
        );

        // Verify all methods exist - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_after, &string::utf8(b"account-key")), 14001); // Original
        assert!(did::test_verification_method_exists(did_document_after, &ed25519_fragment), 14002); // Ed25519
        assert!(did::test_verification_method_exists(did_document_after, &secp256k1_fragment), 14003); // Secp256k1

        // Verify correct relationships
        assert!(did::has_verification_relationship_in_doc(did_document_after, &ed25519_fragment, 1), 14004); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_after, &ed25519_fragment, 4), 14005); // key_agreement
        assert!(did::has_verification_relationship_in_doc(did_document_after, &secp256k1_fragment, 1), 14006); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_after, &secp256k1_fragment, 2), 14007); // capability_invocation
    }

    #[test]
    /// Test validation of verification methods and services
    fun test_validation_functions() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Test is_verification_method_valid_in_doc
        let account_key_fragment = string::utf8(b"account-key");
        let nonexistent_fragment = string::utf8(b"nonexistent");
        
        assert!(did::is_verification_method_valid_in_doc(did_document, &account_key_fragment), 14101);
        assert!(!did::is_verification_method_valid_in_doc(did_document, &nonexistent_fragment), 14102);

        // Add a service and test service validation
        let service_fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");

        did::add_service_entry(&did_signer, service_fragment, service_type, service_endpoint);

        // Test service existence - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_after, &service_fragment), 14103);
        assert!(!did::test_service_exists(did_document_after, &nonexistent_fragment), 14104);
    }

    #[test]
    /// Test DID formatting and parsing edge cases
    fun test_did_formatting_edge_cases() {
        // Test very long identifier
        let long_identifier = string::utf8(b"bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4verylongidentifiertotestlimits123456789");
        let long_did = did::new_did_from_parts(string::utf8(b"rooch"), long_identifier);
        let formatted_long = did::format_did(&long_did);
        let expected_long = string::utf8(b"did:rooch:");
        string::append(&mut expected_long, long_identifier);
        assert!(formatted_long == expected_long, 14201);

        // Test special characters in identifier (should work)
        let special_identifier = string::utf8(b"bc1q-test_123.identifier");
        let special_did = did::new_did_from_parts(string::utf8(b"rooch"), special_identifier);
        let formatted_special = did::format_did(&special_did);
        let expected_special = string::utf8(b"did:rooch:");
        string::append(&mut expected_special, special_identifier);
        assert!(formatted_special == expected_special, 14202);

        // Test different method names
        let custom_method = string::utf8(b"custom");
        let custom_identifier = string::utf8(b"identifier123");
        let custom_did = did::new_did_from_parts(custom_method, custom_identifier);
        let formatted_custom = did::format_did(&custom_did);
        let expected_custom = string::utf8(b"did:custom:identifier123");
        assert!(formatted_custom == expected_custom, 14203);
    }

    #[test]
    /// Test multiple services and verification methods management
    fun test_multiple_services_and_methods() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add multiple verification methods
        let vm_count = 3;
        let i = 0;
        while (i < vm_count) {
            let fragment = string::utf8(b"test-key-");
            let index_str = if (i == 0) { string::utf8(b"0") } 
                           else if (i == 1) { string::utf8(b"1") } 
                           else { string::utf8(b"2") };
            string::append(&mut fragment, index_str);
            
            let method_type = string::utf8(b"Ed25519VerificationKey2020");
            let test_key = if (i == 0) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK") }
                          else if (i == 1) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doL") }
                          else { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doM") };
            let relationships = vector[1u8]; // assertion_method

            did::add_verification_method_entry(&did_signer, fragment, method_type, test_key, relationships);
            i = i + 1;
        };

        // Add multiple services
        let service_count = 3;
        let j = 0;
        while (j < service_count) {
            let service_fragment = string::utf8(b"service-");
            let index_str = if (j == 0) { string::utf8(b"0") }
                           else if (j == 1) { string::utf8(b"1") }
                           else { string::utf8(b"2") };
            string::append(&mut service_fragment, index_str);
            
            let service_type = string::utf8(b"TestService");
            let service_endpoint = string::utf8(b"https://service-");
            string::append(&mut service_endpoint, index_str);
            string::append_utf8(&mut service_endpoint, b".example.com");

            did::add_service_entry(&did_signer, service_fragment, service_type, service_endpoint);
            j = j + 1;
        };

        // Verify all methods and services exist - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        
        // Check verification methods
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"test-key-0")), 14301);
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"test-key-1")), 14302);
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"test-key-2")), 14303);
        
        // Check services
        assert!(did::test_service_exists(did_document_check, &string::utf8(b"service-0")), 14304);
        assert!(did::test_service_exists(did_document_check, &string::utf8(b"service-1")), 14305);
        assert!(did::test_service_exists(did_document_check, &string::utf8(b"service-2")), 14306);
    }

    #[test]
    /// Test comprehensive DID lifecycle - create, modify, query
    fun test_comprehensive_did_lifecycle() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // 1. Verify DID exists
        assert!(did::exists_did_for_address(did_address), 15002);

        // 2. Add verification method
        let vm_fragment = string::utf8(b"backup-key");
        let vm_type = string::utf8(b"Ed25519VerificationKey2020");
        let vm_key = did_test_common::generate_test_ed25519_multibase();
        let vm_relationships = vector[1u8]; // assertion_method (avoid authentication)

        did::add_verification_method_entry(&did_signer, vm_fragment, vm_type, vm_key, vm_relationships);

        // 3. Add service
        let service_fragment = string::utf8(b"api-service");
        let service_type = string::utf8(b"APIService");
        let service_endpoint = string::utf8(b"https://api.example.com");

        did::add_service_entry(&did_signer, service_fragment, service_type, service_endpoint);

        // 4. Verify all components exist - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &vm_fragment), 15003);
        assert!(did::test_service_exists(did_document_check, &service_fragment), 15004);
        assert!(did::has_verification_relationship_in_doc(did_document_check, &vm_fragment, 1), 15006); // assertion_method

        // 5. Modify verification relationships
        did::add_to_verification_relationship_entry(&did_signer, vm_fragment, 4u8); // key_agreement
        
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_after, &vm_fragment, 4), 15007); // key_agreement

        // 6. Update service
        let new_service_type = string::utf8(b"UpdatedAPIService");
        let new_service_endpoint = string::utf8(b"https://api-v2.example.com");
        let property_keys = vector[string::utf8(b"version")];
        let property_values = vector[string::utf8(b"2.0")];

        did::update_service_entry(
            &did_signer,
            service_fragment,
            new_service_type,
            new_service_endpoint,
            property_keys,
            property_values
        );

        // 7. Query and verify final state - use DID address
        let final_did_document = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(final_did_document, &service_fragment), 15008);
        
        // 8. Test controller mapping
        let controller_did = did::new_rooch_did_by_address(_creator_address);
        let controlled_dids = did::get_dids_by_controller(controller_did);
        assert!(vector::length(&controlled_dids) == 1, 15009);

        // 9. Cleanup - remove method and service
        did::remove_from_verification_relationship_entry(&did_signer, vm_fragment, 4u8); // remove key_agreement
        did::remove_verification_method_entry(&did_signer, vm_fragment);
        did::remove_service_entry(&did_signer, service_fragment);

        // 10. Verify cleanup - use DID address
        let final_did_document_cleaned = did::get_did_document_by_address(did_address);
        assert!(!did::test_verification_method_exists(final_did_document_cleaned, &vm_fragment), 15010);
        assert!(!did::test_service_exists(final_did_document_cleaned, &service_fragment), 15011);
    }

    #[test]
    /// Test performance with large number of verification methods
    fun test_performance_many_verification_methods() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add multiple verification methods for performance testing
        let method_count = 10;
        let i = 0;
        while (i < method_count) {
            let fragment = string::utf8(b"perf-key-");
            let index_bytes = if (i < 10) {
                vector[48 + i] // ASCII '0' + i for single digits
            } else {
                vector[48 + 1, 48 + (i - 10)] // '1' + digit for 10+
            };
            string::append_utf8(&mut fragment, index_bytes);
            
            let method_type = string::utf8(b"Ed25519VerificationKey2020");
            
            // Generate different keys for each method
            let test_key = if (i == 0) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do0") }
                          else if (i == 1) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do1") }
                          else if (i == 2) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do2") }
                          else if (i == 3) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do3") }
                          else if (i == 4) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do4") }
                          else if (i == 5) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do5") }
                          else if (i == 6) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do6") }
                          else if (i == 7) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do7") }
                          else if (i == 8) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do8") }
                          else { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do9") };
                          
            let relationships = vector[1u8]; // assertion_method

            did::add_verification_method_entry(&did_signer, fragment, method_type, test_key, relationships);
            i = i + 1;
        };

        // Verify all methods were added - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        let j = 0;
        while (j < method_count) {
            let fragment = string::utf8(b"perf-key-");
            let index_bytes = if (j < 10) {
                vector[48 + j] // ASCII '0' + j for single digits
            } else {
                vector[48 + 1, 48 + (j - 10)] // '1' + digit for 10+
            };
            string::append_utf8(&mut fragment, index_bytes);
            
            let error_code = 15100 + (j as u64);
            assert!(did::test_verification_method_exists(did_document_check, &fragment), error_code);
            j = j + 1;
        };
    }

    #[test]
    /// Test cross-cutting functionality integration
    fun test_cross_cutting_integration() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Test 1: DID String Parsing and Formatting Integration
        let test_did_string = string::utf8(b"did:rooch:");
        let test_bech32 = moveos_std::address::to_bech32_string(did_address);
        string::append(&mut test_did_string, test_bech32);
        
        let parsed_did = did::parse_did_string(&test_did_string);
        let formatted_did = did::format_did(&parsed_did);
        assert!(formatted_did == test_did_string, 15201);

        // Test 2: Controller Mapping Integration
        let controller_did = did::new_rooch_did_by_address(_creator_address);
        let controlled_dids = did::get_dids_by_controller(controller_did);
        assert!(vector::length(&controlled_dids) == 1, 15202);

        // Test 3: Verification Method and Service Integration
        let vm_fragment = string::utf8(b"integration-key");
        let vm_type = string::utf8(b"Ed25519VerificationKey2020");
        let vm_key = did_test_common::generate_test_ed25519_multibase();
        let vm_relationships = vector[0u8, 1u8, 2u8]; // authentication, assertion_method, capability_invocation

        did::add_verification_method_entry(&did_signer, vm_fragment, vm_type, vm_key, vm_relationships);

        let service_fragment = string::utf8(b"integration-service");
        let service_type = string::utf8(b"IntegrationService");
        let service_endpoint = string::utf8(b"https://integration.example.com");

        did::add_service_entry(&did_signer, service_fragment, service_type, service_endpoint);

        // Test 4: Permission Integration - Switch to new verification method for capability_invocation
        let vm_pk_bytes_opt = multibase_codec::decode(&vm_key);
        let vm_pk_bytes = option::destroy_some(vm_pk_bytes_opt);
        let vm_auth_key = session_key::ed25519_public_key_to_authentication_key(&vm_pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(vm_auth_key));

        // Should be able to add services with capabilityInvocation permission
        let service2_fragment = string::utf8(b"integration-service-2");
        let service2_type = string::utf8(b"IntegrationService2");
        let service2_endpoint = string::utf8(b"https://integration2.example.com");

        did::add_service_entry(&did_signer, service2_fragment, service2_type, service2_endpoint);

        // Test 5: Query Integration - Verify all components - use DID address
        let final_did_document = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(final_did_document, &vm_fragment), 15203);
        assert!(did::test_service_exists(final_did_document, &service_fragment), 15204);
        assert!(did::test_service_exists(final_did_document, &service2_fragment), 15205);
        assert!(did::has_verification_relationship_in_doc(final_did_document, &vm_fragment, 1), 15206); // assertion_method
        assert!(did::has_verification_relationship_in_doc(final_did_document, &vm_fragment, 2), 15207); // capability_invocation
    }

    #[test]
    /// Test system resilience and recovery scenarios
    fun test_system_resilience() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Scenario 1: Recovery after adding and removing multiple methods
        let recovery_count = 5;
        let i = 0;
        while (i < recovery_count) {
            // Add method
            let fragment = string::utf8(b"recovery-");
            let index_bytes = vector[48 + i]; // ASCII '0' + i
            string::append_utf8(&mut fragment, index_bytes);
            
            let method_type = string::utf8(b"Ed25519VerificationKey2020");
            let test_key = if (i == 0) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2d00") }
                          else if (i == 1) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2d01") }
                          else if (i == 2) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2d02") }
                          else if (i == 3) { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2d03") }
                          else { string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2d04") };
            let relationships = vector[1u8]; // assertion_method

            did::add_verification_method_entry(&did_signer, fragment, method_type, test_key, relationships);
            
            // Immediately remove it
            did::remove_verification_method_entry(&did_signer, fragment);
            
            i = i + 1;
        };

        // Scenario 2: Verify DID is still functional after operations
        let final_fragment = string::utf8(b"final-recovery-key");
        let final_type = string::utf8(b"Ed25519VerificationKey2020");
        let final_key = string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2dFin");
        let final_relationships = vector[1u8]; // assertion_method (avoid authentication)

        did::add_verification_method_entry(&did_signer, final_fragment, final_type, final_key, final_relationships);

        // Scenario 3: Verify DID document integrity - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &final_fragment), 15301);
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"account-key")), 15302); // Original still exists
        assert!(did::has_verification_relationship_in_doc(did_document_check, &final_fragment, 1), 15304); // assertion_method

        // Scenario 4: Controller mapping still works
        let controller_did = did::new_rooch_did_by_address(_creator_address);
        let controlled_dids = did::get_dids_by_controller(controller_did);
        assert!(vector::length(&controlled_dids) == 1, 15305);
    }

    #[test]
    /// Test edge cases and boundary conditions
    fun test_edge_cases_and_boundaries() {
        did_test_common::init_test_framework();

        // Test 1: Empty string handling
        assert!(!did::exists_did_document_by_identifier(string::utf8(b"")), 15401);

        // Test 2: Special address cases
        assert!(!did::exists_did_for_address(@0x0), 15402);
        assert!(!did::exists_did_for_address(@0x1), 15403);

        // Test 3: Very long DID identifiers
        let long_identifier = string::utf8(b"thisisaverylongidentifierthatcontainsmanycharsfortestingpurposes123456789abcdefghijklmnopqrstuvwxyz");
        let long_did = did::new_did_from_parts(string::utf8(b"test"), long_identifier);
        let formatted_long = did::format_did(&long_did);
        assert!(string::length(&formatted_long) > 100, 15404);

        // Test 4: Special characters in identifiers
        let special_chars_identifier = string::utf8(b"user-123_test.example-with-many.special-chars");
        let special_did = did::new_did_from_parts(string::utf8(b"test"), special_chars_identifier);
        let formatted_special = did::format_did(&special_did);
        let expected_special = string::utf8(b"did:test:user-123_test.example-with-many.special-chars");
        assert!(formatted_special == expected_special, 15405);

        // Test 5: Multiple colons in identifiers (valid per DID spec)
        let multi_colon_identifier = string::utf8(b"namespace:type:id:subtype:version:1.0");
        let multi_colon_did = did::new_did_from_parts(string::utf8(b"test"), multi_colon_identifier);
        let formatted_multi_colon = did::format_did(&multi_colon_did);
        let expected_multi_colon = string::utf8(b"did:test:namespace:type:id:subtype:version:1.0");
        assert!(formatted_multi_colon == expected_multi_colon, 15406);

        // Test 6: Controller queries with different DID types
        let rooch_controller = did::new_did_from_parts(string::utf8(b"rooch"), string::utf8(b"test123"));
        let key_controller = did::new_did_from_parts(string::utf8(b"key"), string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"));
        
        let rooch_controlled = did::get_dids_by_controller(rooch_controller);
        let key_controlled = did::get_dids_by_controller(key_controller);
        
        assert!(vector::length(&rooch_controlled) == 0, 15407); // No DIDs initially
        assert!(vector::length(&key_controlled) == 0, 15408); // No DIDs initially
    }
} 