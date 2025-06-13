// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Permission & Authorization functionality
/// Covers session key authentication, capability permissions, and authorization checks
module rooch_framework::did_permission_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::option;
    use moveos_std::account;
    use moveos_std::multibase_codec;

    // ========================================
    // Test Category 4: Permission & Authorization Tests
    // ========================================

    #[test]
    #[expected_failure(abort_code = 26, location = rooch_framework::did)] // ErrorInsufficientPermission
    /// Test authorization failure when verification method lacks capabilityDelegation permission
    fun test_authorization_capability_delegation_invalid() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use basic setup to avoid session key conflicts
        let (creator_signer, _creator_address, creator_public_key_multibase) = did_test_common::setup_did_test_basic();
        let did_object_id = did::create_did_object_for_self(&creator_signer, creator_public_key_multibase);
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add a verification method with only assertion_method permission (no capabilityDelegation)
        let fragment = string::utf8(b"limited-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[0u8, 1u8]; // authentication, assertion_method only

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            test_key,
            relationships
        );

        // Now create a fresh test framework and switch to the limited key's session key
        let limited_pk_bytes_opt = multibase_codec::decode(&test_key);
        let limited_pk_bytes = option::destroy_some(limited_pk_bytes_opt);
        let limited_auth_key = session_key::ed25519_public_key_to_authentication_key(&limited_pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(limited_auth_key));

        // Try to add another verification method - should fail due to insufficient permission
        let another_fragment = string::utf8(b"another-key");
        let another_method_type = string::utf8(b"Ed25519VerificationKey2020");
        let another_test_key = string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doX"); // Different key
        let another_relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(
            &did_signer,
            another_fragment,
            another_method_type,
            another_test_key,
            another_relationships
        );
    }

    #[test]
    #[expected_failure(abort_code = 26, location = rooch_framework::did)] // ErrorInsufficientPermission  
    /// Test authorization failure when verification method lacks capabilityInvocation permission
    fun test_authorization_capability_invocation_invalid() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use basic setup to avoid session key conflicts
        let (creator_signer, _creator_address, creator_public_key_multibase) = did_test_common::setup_did_test_basic();
        let did_object_id = did::create_did_object_for_self(&creator_signer, creator_public_key_multibase);
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add a verification method with only assertion_method permission (no capabilityInvocation)
        let fragment = string::utf8(b"limited-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[0u8, 1u8]; // authentication, assertion_method only

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            test_key,
            relationships
        );

        // Now create a fresh test framework and switch to the limited key's session key
        let limited_pk_bytes_opt = multibase_codec::decode(&test_key);
        let limited_pk_bytes = option::destroy_some(limited_pk_bytes_opt);
        let limited_auth_key = session_key::ed25519_public_key_to_authentication_key(&limited_pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(limited_auth_key));

        // Try to add a service - should fail due to insufficient permission
        let service_fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");

        did::add_service_entry(&did_signer, service_fragment, service_type, service_endpoint);
    }

    #[test]
    #[expected_failure(abort_code = 25, location = rooch_framework::did)] // ErrorSessionKeyNotFound
    /// Test authorization failure when session key is not found in authentication methods
    fun test_authorization_session_key_not_found() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use standard setup with DID creation
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Create a fresh test framework and setup a completely different session key
        let random_key = did_test_common::generate_test_ed25519_multibase();
        let random_pk_bytes_opt = multibase_codec::decode(&random_key);
        let random_pk_bytes = option::destroy_some(random_pk_bytes_opt);
        let random_auth_key = session_key::ed25519_public_key_to_authentication_key(&random_pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(random_auth_key));

        // Try to add verification method - should fail because session key not found
        let fragment = string::utf8(b"new-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            test_key,
            relationships
        );
    }


    #[test]
    /// Test successful authorization with valid capabilityDelegation permission
    fun test_authorization_capability_delegation_valid() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // The setup already configured the account-key session key which has capabilityDelegation
        // Add verification method - should succeed with valid authorization
        let fragment = string::utf8(b"new-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            test_key,
            relationships
        );

        // Verify method was added successfully
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_after, &fragment), 10001);
    }

    #[test]
    /// Test successful authorization with valid capabilityInvocation permission
    fun test_authorization_capability_invocation_valid() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Setup session key for authorization (account-key has capabilityInvocation)
        let pk_bytes_opt = multibase_codec::decode(&_creator_public_key_multibase);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));

        // Add service - should succeed with valid authorization
        let fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");

        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);

        // Verify service was added successfully
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_after, &fragment), 10101);
    }

    #[test]
    /// Test session key to verification method mapping for Ed25519 keys
    fun test_find_verification_method_by_session_key_ed25519() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Setup session key for authorization with account-key
        let pk_bytes_opt = multibase_codec::decode(&_creator_public_key_multibase);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));

        // Add Ed25519 verification method
        let ed25519_fragment = string::utf8(b"ed25519-key");
        let ed25519_type = string::utf8(b"Ed25519VerificationKey2020");
        let ed25519_key = did_test_common::generate_test_ed25519_multibase();
        let ed25519_relationships = vector[0u8]; // authentication

        did::add_verification_method_entry(
            &did_signer,
            ed25519_fragment,
            ed25519_type,
            ed25519_key,
            ed25519_relationships
        );

        // Now test using the Ed25519 key as session key
        let ed25519_pk_bytes_opt = multibase_codec::decode(&ed25519_key);
        let ed25519_pk_bytes = option::destroy_some(ed25519_pk_bytes_opt);
        let ed25519_auth_key = session_key::ed25519_public_key_to_authentication_key(&ed25519_pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(ed25519_auth_key));

        // Verify the Ed25519 key can be found by session key
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_check, &ed25519_fragment, 0), 10201); // authentication
    }

    #[test]
    /// Test session key to verification method mapping for Secp256k1 keys
    fun test_find_verification_method_by_session_key_secp256k1() {
        use rooch_framework::session_key;
        use rooch_framework::auth_validator;
        
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Setup session key for authorization with account-key
        let pk_bytes_opt = multibase_codec::decode(&creator_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), 9001);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));

        // Add another Secp256k1 verification method
        let secp256k1_fragment = string::utf8(b"secp256k1-key");
        let secp256k1_type = string::utf8(b"EcdsaSecp256k1VerificationKey2019");
        let secp256k1_key = did_test_common::generate_test_secp256k1_multibase(); // Generate valid test key
        let secp256k1_relationships = vector[0u8]; // authentication

        did::add_verification_method_entry(
            &did_signer,
            secp256k1_fragment,
            secp256k1_type,
            secp256k1_key,
            secp256k1_relationships
        );

        // Now test using the Secp256k1 key as session key
        let secp256k1_pk_bytes_opt = multibase_codec::decode(&secp256k1_key);
        assert!(option::is_some(&secp256k1_pk_bytes_opt), 9002);
        let secp256k1_pk_bytes = option::destroy_some(secp256k1_pk_bytes_opt);
        let secp256k1_auth_key = session_key::secp256k1_public_key_to_authentication_key(&secp256k1_pk_bytes);
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(secp256k1_auth_key));

        // Verify the Secp256k1 key can be found by session key
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_check, &secp256k1_fragment, 0), 10302); // authentication
    }

    #[test]
    /// Test permission hierarchy for capabilityDelegation operations
    fun test_permission_hierarchy_capability_delegation() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // The setup already configured the account-key session key with capabilityDelegation

        // Test operations that require capabilityDelegation
        // 1. Add verification method - requires capabilityDelegation
        let fragment1 = string::utf8(b"test-key-1");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key1 = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(&did_signer, fragment1, method_type, test_key1, relationships);

        // 2. Remove verification method - requires capabilityDelegation
        did::remove_verification_method_entry(&did_signer, fragment1);

        // 3. Add to verification relationship - requires capabilityDelegation
        let fragment2 = string::utf8(b"test-key-2");
        let test_key2 = string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doL");
        did::add_verification_method_entry(&did_signer, fragment2, method_type, test_key2, relationships);
        
        did::add_to_verification_relationship_entry(&did_signer, fragment2, 4u8); // key_agreement

        // 4. Remove from verification relationship - requires capabilityDelegation
        did::remove_from_verification_relationship_entry(&did_signer, fragment2, 4u8); // key_agreement

        // Verify operations completed successfully
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &fragment2), 10401);
        assert!(!did::test_verification_method_exists(did_document_check, &fragment1), 10402); // Removed
    }

    #[test]
    /// Test permission hierarchy for capabilityInvocation operations
    fun test_permission_hierarchy_capability_invocation() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // The setup already configured the account-key session key with capabilityInvocation

        // Test operations that require capabilityInvocation
        // 1. Add service - requires capabilityInvocation
        let service_fragment1 = string::utf8(b"test-service-1");
        let service_type = string::utf8(b"TestService");
        let service_endpoint1 = string::utf8(b"https://test1.example.com");

        did::add_service_entry(&did_signer, service_fragment1, service_type, service_endpoint1);

        // 2. Update service - requires capabilityInvocation
        let new_service_type = string::utf8(b"UpdatedTestService");
        let new_service_endpoint = string::utf8(b"https://updated.example.com");
        let property_keys = vector[string::utf8(b"version")];
        let property_values = vector[string::utf8(b"2.0")];

        did::update_service_entry(
            &did_signer,
            service_fragment1,
            new_service_type,
            new_service_endpoint,
            property_keys,
            property_values
        );

        // 3. Remove service - requires capabilityInvocation
        did::remove_service_entry(&did_signer, service_fragment1);

        // Verify service operations completed successfully
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(!did::test_service_exists(did_document_check, &service_fragment1), 10501); // Service removed
    }

    #[test]
    /// Test multiple authentication methods with different permissions
    fun test_multiple_authentication_methods_permissions() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // The setup already configured the account-key session key with all permissions

        // Add verification method with capabilityDelegation permission
        let delegation_fragment = string::utf8(b"delegation-key");
        let delegation_type = string::utf8(b"Ed25519VerificationKey2020");
        let delegation_key = did_test_common::generate_test_ed25519_multibase();
        let delegation_relationships = vector[0u8, 3u8]; // authentication, capability_delegation

        did::add_verification_method_entry(
            &did_signer,
            delegation_fragment,
            delegation_type,
            delegation_key,
            delegation_relationships
        );

        // Add verification method with capabilityInvocation permission
        let invocation_fragment = string::utf8(b"invocation-key");
        let invocation_type = string::utf8(b"Ed25519VerificationKey2020");
        let invocation_key = did_test_common::generate_test_ed25519_multibase();
        let invocation_relationships = vector[0u8, 2u8]; // authentication, capability_invocation

        did::add_verification_method_entry(
            &did_signer,
            invocation_fragment,
            invocation_type,
            invocation_key,
            invocation_relationships
        );

        // Verify all methods were added
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &delegation_fragment), 10601);
        assert!(did::test_verification_method_exists(did_document_check, &invocation_fragment), 10602);

        // Verify correct permissions
        assert!(did::has_verification_relationship_in_doc(did_document_check, &delegation_fragment, 3), 10603); // capability_delegation
        assert!(did::has_verification_relationship_in_doc(did_document_check, &invocation_fragment, 2), 10604); // capability_invocation
    }
} 