// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Verification Method Management functionality
/// Covers adding, removing verification methods and managing verification relationships
module rooch_framework::did_verification_method_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use moveos_std::account;

    // ========================================
    // Test Category 2: Verification Method Management Tests  
    // ========================================

    #[test]
    /// Test adding Ed25519 verification method successfully
    fun test_add_verification_method_ed25519() {        
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        
        // Create a signer for the DID address
        let did_signer = account::create_signer_for_testing(did_address);

        // Add new Ed25519 verification method - use a different working Ed25519 key
        let new_ed25519_key = did_test_common::generate_test_ed25519_multibase();
        let fragment = string::utf8(b"ed25519-key-1");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let relationships = vector[0u8, 1u8]; // assertion_method only (not authentication to avoid session key registration)

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            new_ed25519_key,
            relationships
        );

        // Verify method was added - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_after, &fragment), 8001);
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 1), 8003); // assertion_method
    }

    #[test]
    /// Test adding Secp256k1 verification method successfully
    fun test_add_verification_method_secp256k1() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add new Secp256k1 verification method
        let new_secp256k1_key = did_test_common::generate_test_secp256k1_multibase(); // Different key
        let fragment = string::utf8(b"secp256k1-key-1");
        let method_type = string::utf8(b"EcdsaSecp256k1VerificationKey2019");
        let relationships = vector[0u8, 2u8, 3u8]; // capability_invocation, capability_delegation

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            new_secp256k1_key,
            relationships
        );

        // Verify method was added - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_after, &fragment), 8101);
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 2), 8102); // capability_invocation
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 3), 8103); // capability_delegation
    }

    #[test]
    #[expected_failure(abort_code = 5, location = rooch_framework::did)] // ErrorVerificationMethodAlreadyExists
    /// Test adding verification method with duplicate fragment fails
    fun test_add_verification_method_already_exists() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Try to add method with same fragment as existing account-key
        let fragment = string::utf8(b"account-key"); // This already exists from DID creation
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let new_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[0u8]; // authentication

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            new_key,
            relationships
        );
    }

    #[test]
    /// Test removing verification method successfully
    fun test_remove_verification_method_success() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add a verification method first
        let fragment = string::utf8(b"test-key");
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

        // Verify method exists - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &fragment), 8201);

        // Remove the verification method
        did::remove_verification_method_entry(&did_signer, fragment);

        // Verify method was removed - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(!did::test_verification_method_exists(did_document_after, &fragment), 8202);
    }

    #[test]
    #[expected_failure(abort_code = 4, location = rooch_framework::did)] // ErrorVerificationMethodNotFound
    /// Test removing non-existent verification method fails
    fun test_remove_verification_method_not_found() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Try to remove non-existent method
        let nonexistent_fragment = string::utf8(b"nonexistent-key");
        did::remove_verification_method_entry(&did_signer, nonexistent_fragment);
    }

    #[test]
    /// Test adding method to verification relationship
    fun test_add_to_verification_relationship_success() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add a verification method first without key_agreement relationship
        let fragment = string::utf8(b"test-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8]; // assertion_method only

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            test_key,
            relationships
        );

        // Verify key_agreement is not set initially - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(!did::has_verification_relationship_in_doc(did_document_check, &fragment, 4), 8301); // key_agreement

        // Add to key_agreement relationship
        did::add_to_verification_relationship_entry(&did_signer, fragment, 4u8); // key_agreement

        // Verify key_agreement relationship was added - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 4), 8302); // key_agreement
    }

    #[test]
    /// Test removing method from verification relationship
    fun test_remove_from_verification_relationship_success() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add a verification method with multiple relationships
        let fragment = string::utf8(b"test-key");
        let method_type = string::utf8(b"Ed25519VerificationKey2020");
        let test_key = did_test_common::generate_test_ed25519_multibase();
        let relationships = vector[1u8, 4u8]; // assertion_method, key_agreement

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            test_key,
            relationships
        );

        // Verify both relationships exist - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_check, &fragment, 1), 8401); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_check, &fragment, 4), 8402); // key_agreement

        // Remove from key_agreement relationship
        did::remove_from_verification_relationship_entry(&did_signer, fragment, 4u8); // key_agreement

        // Verify key_agreement was removed but assertion_method remains - use DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 1), 8403); // assertion_method still exists
        assert!(!did::has_verification_relationship_in_doc(did_document_after, &fragment, 4), 8404); // key_agreement removed
    }

    #[test]
    #[expected_failure(abort_code = 9, location = rooch_framework::did)] // ErrorInvalidVerificationRelationship
    /// Test adding method to invalid verification relationship fails
    fun test_add_to_verification_relationship_invalid_type() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Try to add to invalid relationship type (255 is not a valid relationship)
        let fragment = string::utf8(b"account-key"); // Use existing fragment
        did::add_to_verification_relationship_entry(&did_signer, fragment, 255u8);
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
    /// Test validation of verification methods and services
    fun test_validation_functions() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        let did_document_check = did::get_did_document_by_address(did_address);

        // Test is_verification_method_valid_in_doc
        let account_key_fragment = string::utf8(b"account-key");
        let nonexistent_fragment = string::utf8(b"nonexistent");
        
        assert!(did::is_verification_method_valid_in_doc(did_document_check, &account_key_fragment), 14101);
        assert!(!did::is_verification_method_valid_in_doc(did_document_check, &nonexistent_fragment), 14102);
    }

    #[test]
    /// Test Ed25519 and Secp256k1 key types in same DID document
    fun test_mixed_key_types_in_did() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add Ed25519 verification method - use a working Ed25519 key and avoid authentication relationship
        let ed25519_fragment = string::utf8(b"ed25519-key");
        let ed25519_type = string::utf8(b"Ed25519VerificationKey2020");
        let ed25519_key = did_test_common::generate_test_ed25519_multibase();
        let ed25519_relationships = vector[0u8, 1u8, 4u8]; // assertion_method, key_agreement (avoid authentication)

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
        let secp256k1_relationships = vector[0u8, 1u8, 2u8]; // assertion_method, capability_invocation

        did::add_verification_method_entry(
            &did_signer,
            secp256k1_fragment,
            secp256k1_type,
            secp256k1_key,
            secp256k1_relationships
        );

        // Verify all methods exist - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"account-key")), 14001); // Original
        assert!(did::test_verification_method_exists(did_document_check, &ed25519_fragment), 14002); // Ed25519
        assert!(did::test_verification_method_exists(did_document_check, &secp256k1_fragment), 14003); // Secp256k1

        // Verify correct relationships
        assert!(did::has_verification_relationship_in_doc(did_document_check, &ed25519_fragment, 1), 14006); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_check, &ed25519_fragment, 4), 14005); // key_agreement
        assert!(did::has_verification_relationship_in_doc(did_document_check, &secp256k1_fragment, 1), 14006); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_check, &secp256k1_fragment, 2), 14007); // capability_invocation
    }

    #[test]
    /// Test multiple verification methods management
    fun test_multiple_verification_methods() {
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

        // Verify all methods exist - use DID address
        let did_document_check = did::get_did_document_by_address(did_address);
        
        // Check verification methods
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"test-key-0")), 14301);
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"test-key-1")), 14302);
        assert!(did::test_verification_method_exists(did_document_check, &string::utf8(b"test-key-2")), 14303);
    }

    #[test]
    /// Test adding ECDSA R1 verification method successfully
    fun test_add_verification_method_ecdsa_r1() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add ECDSA R1 verification method
        let fragment = string::utf8(b"ecdsa-r1-key");
        let method_type = string::utf8(b"EcdsaSecp256r1VerificationKey2019");
        let ecdsa_r1_key = did_test_common::generate_test_ecdsa_r1_multibase();
        let relationships = vector[1u8, 2u8]; // assertion_method, capability_invocation

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            ecdsa_r1_key,
            relationships
        );

        // Verify method was added
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_after, &fragment), 15001);
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 1), 15002); // assertion_method
        assert!(did::has_verification_relationship_in_doc(did_document_after, &fragment, 2), 15003); // capability_invocation
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorVerificationMethodAlreadyExists, location = rooch_framework::did)] // ErrorVerificationMethodAlreadyExists
    /// Test adding duplicate ECDSA R1 verification method fails
    fun test_add_duplicate_ecdsa_r1_verification_method() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add first ECDSA R1 verification method
        let fragment = string::utf8(b"ecdsa-r1-key");
        let method_type = string::utf8(b"EcdsaSecp256r1VerificationKey2019");
        let ecdsa_r1_key = did_test_common::generate_test_ecdsa_r1_multibase();
        let relationships = vector[1u8]; // assertion_method

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            ecdsa_r1_key,
            relationships
        );

        // Try to add the same verification method again
        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            ecdsa_r1_key,
            relationships
        );
    }

    #[test]
    /// Test removing ECDSA R1 verification method
    fun test_remove_ecdsa_r1_verification_method() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add ECDSA R1 verification method
        let fragment = string::utf8(b"ecdsa-r1-key");
        let method_type = string::utf8(b"EcdsaSecp256r1VerificationKey2019");
        let ecdsa_r1_key = did_test_common::generate_test_ecdsa_r1_multibase();
        let relationships = vector[1u8, 2u8]; // assertion_method, capability_invocation

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            ecdsa_r1_key,
            relationships
        );

        // Verify method exists
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_verification_method_exists(did_document_check, &fragment), 16001);

        // Remove the verification method
        did::remove_verification_method_entry(&did_signer, fragment);

        // Verify method was removed
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(!did::test_verification_method_exists(did_document_after, &fragment), 16002);
    }

    #[test]
    /// Test adding and removing ECDSA R1 verification method from relationships
    fun test_ecdsa_r1_verification_relationships() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add ECDSA R1 verification method with initial relationships
        let fragment = string::utf8(b"ecdsa-r1-key");
        let method_type = string::utf8(b"EcdsaSecp256r1VerificationKey2019");
        let ecdsa_r1_key = did_test_common::generate_test_ecdsa_r1_multibase();
        let relationships = vector[1u8]; // assertion_method only

        did::add_verification_method_entry(
            &did_signer,
            fragment,
            method_type,
            ecdsa_r1_key,
            relationships
        );

        // Verify initial relationships
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_check, &fragment, 1), 17001); // assertion_method
        assert!(!did::has_verification_relationship_in_doc(did_document_check, &fragment, 2), 17002); // capability_invocation

        // Add capability_invocation relationship
        did::add_to_verification_relationship_entry(&did_signer, fragment, 2u8);

        // Verify new relationship was added
        let did_document_after_add = did::get_did_document_by_address(did_address);
        assert!(did::has_verification_relationship_in_doc(did_document_after_add, &fragment, 2), 17003); // capability_invocation

        // Remove capability_invocation relationship
        did::remove_from_verification_relationship_entry(&did_signer, fragment, 2u8);

        // Verify relationship was removed
        let did_document_after_remove = did::get_did_document_by_address(did_address);
        assert!(!did::has_verification_relationship_in_doc(did_document_after_remove, &fragment, 2), 17004); // capability_invocation
    }
} 