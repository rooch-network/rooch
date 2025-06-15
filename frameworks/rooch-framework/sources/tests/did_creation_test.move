// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Object Creation functionality
/// Covers DID creation, string parsing, registry initialization, and basic validation
module rooch_framework::did_creation_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::option;
    use std::vector;
    use moveos_std::multibase_codec;

    // ========================================
    // Test Category 1: DID Object Creation Tests
    // ========================================

    #[test]
    /// Test successful DID creation for self using account key only
    /// This test verifies the core DID creation functionality:
    /// 1. DID does not exist before creation
    /// 2. DID creation succeeds with valid parameters
    /// 3. DID exists after creation and can be queried
    /// 4. DID registry is properly updated
    fun test_create_did_for_self_success() {
        let (creator_signer, creator_address, creator_public_key_multibase, did_object_id) = 
            did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document to find the real DID address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        
        // Verify DID exists at the correct address (not creator_address, but the new DID address)
        assert!(did::exists_did_for_address(did_address), 1002);
        
        // Verify registry was initialized
        let empty_dids = did::get_dids_by_controller_string(string::utf8(b"did:rooch:nonexistent"));
        assert!(vector::length(&empty_dids) == 0, 1003);

        // Verify DID identifier format
        let did_identifier = did::doc_id(did_document);
        let formatted_did = did::format_did(did_identifier);
        
        // Build expected DID string using the actual DID address
        let expected_did_string = string::utf8(b"did:rooch:");
        let did_bech32 = moveos_std::address::to_bech32_string(did_address);
        string::append(&mut expected_did_string, did_bech32);
        
        // Verify DID format is correct
        assert!(formatted_did == expected_did_string, 1004);
        
        // Verify DID document properties
        let controllers = did::doc_controllers(did_document);
        assert!(vector::length(controllers) == 1, 1005);
        let expected_controller = did::new_rooch_did_by_address(creator_address);
        assert!(*vector::borrow(controllers, 0) == expected_controller, 1006);
        
        // Verify account-key verification method exists
        let account_key_fragment = string::utf8(b"account-key");
        let vm_opt = did::doc_verification_method(did_document, &account_key_fragment);
        assert!(option::is_some(&vm_opt), 1007);
        
        let vm = option::destroy_some(vm_opt);
        let vm_type = did::verification_method_type(&vm);
        assert!(*vm_type == string::utf8(b"EcdsaSecp256k1VerificationKey2019"), 1008);
        
        let vm_pubkey = did::verification_method_public_key_multibase(&vm);
        assert!(*vm_pubkey == creator_public_key_multibase, 1009);
        
        // Verify verification relationships
        let auth_methods = did::doc_authentication_methods(did_document);
        assert!(vector::contains(auth_methods, &account_key_fragment), 1010);
        
        let assertion_methods = did::doc_assertion_methods(did_document);
        assert!(vector::contains(assertion_methods, &account_key_fragment), 1011);
        
        let capability_invocation = did::doc_capability_invocation_methods(did_document);
        assert!(vector::contains(capability_invocation, &account_key_fragment), 1012);
        
        let capability_delegation = did::doc_capability_delegation_methods(did_document);
        assert!(vector::contains(capability_delegation, &account_key_fragment), 1013);
        
        // The did address not creator address
        assert!(did_address != creator_address, 1016);
        
        // Clean up compiler warnings - assign each variable separately
        let _ = creator_signer;
        let _ = did_object_id;
    }

    #[test]
    /// Test DID string parsing functionality
    /// Verifies correct parsing and formatting of DID strings
    fun test_did_string_parsing() {
        // Test valid did:rooch format
        let rooch_did_string = string::utf8(b"did:rooch:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4");
        let parsed_rooch_did = did::parse_did_string(&rooch_did_string);
        let formatted_rooch_did = did::format_did(&parsed_rooch_did);
        assert!(formatted_rooch_did == rooch_did_string, 2001);

        // Test valid did:key format
        let key_did_string = string::utf8(b"did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        let parsed_key_did = did::parse_did_string(&key_did_string);
        let formatted_key_did = did::format_did(&parsed_key_did);
        assert!(formatted_key_did == key_did_string, 2002);

        // Test creating DID from parts
        let method = string::utf8(b"rooch");
        let identifier = string::utf8(b"bc1qtest123");
        let created_did = did::new_did_from_parts(method, identifier);
        let expected_did_string = string::utf8(b"did:rooch:bc1qtest123");
        let formatted_created_did = did::format_did(&created_did);
        assert!(formatted_created_did == expected_did_string, 2003);
    }

    #[test]
    /// Test DID registry initialization and basic queries
    /// Verifies the DID registry can be initialized and responds correctly to queries
    fun test_did_registry_initialization() {
        // Initialize the entire framework including DID registry
        did_test_common::init_test_framework();

        // Test empty controller query
        let empty_controller_did = string::utf8(b"did:rooch:nonexistent");
        let dids_for_empty_controller = did::get_dids_by_controller_string(empty_controller_did);
        assert!(vector::length(&dids_for_empty_controller) == 0, 3001);

        // Test DID existence check for non-existent DID
        let test_identifier = string::utf8(b"bc1qnonexistent");
        assert!(!did::exists_did_document_by_identifier(test_identifier), 3002);

        // Test address-based DID existence check
        let test_address = @0x999;
        assert!(!did::exists_did_for_address(test_address), 3003);
    }

    #[test]
    /// Test mock session key and Bitcoin address setup
    /// Demonstrates how to use the mock functions for testing DID operations
    fun test_mock_session_key_and_bitcoin_address() {
        use rooch_framework::auth_validator;
        use rooch_framework::session_key;
        
        // Initialize the framework
        did_test_common::init_test_framework();

        // Generate test public key and derive authentication key for session
        let test_ed25519_key = did_test_common::generate_test_ed25519_multibase();
        let pk_bytes_opt = multibase_codec::decode(&test_ed25519_key);
        assert!(option::is_some(&pk_bytes_opt), 5001);
        let pk_bytes = option::destroy_some(pk_bytes_opt);
        
        // Derive authentication key that would be used as session key
        let auth_key = session_key::ed25519_public_key_to_authentication_key(&pk_bytes);
        
        // Set up mock session key in transaction context
        auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));
        
        // Verify the mock session key can be retrieved
        let retrieved_session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(option::is_some(&retrieved_session_key_opt), 5002);
        let retrieved_session_key = option::extract(&mut retrieved_session_key_opt);
        assert!(retrieved_session_key == auth_key, 5003);
        
        // Verify Bitcoin address can be retrieved
        let bitcoin_address_opt = auth_validator::get_bitcoin_address_from_ctx_option();
        assert!(option::is_some(&bitcoin_address_opt), 5004);
        let bitcoin_address = option::extract(&mut bitcoin_address_opt);
        assert!(!rooch_framework::bitcoin_address::is_empty(&bitcoin_address), 5005);
        
        // Test with random Bitcoin address
        auth_validator::set_random_tx_validate_result_for_testing(option::some(auth_key));
        let random_bitcoin_address_opt = auth_validator::get_bitcoin_address_from_ctx_option();
        assert!(option::is_some(&random_bitcoin_address_opt), 5006);
        let random_bitcoin_address = option::extract(&mut random_bitcoin_address_opt);
        assert!(!rooch_framework::bitcoin_address::is_empty(&random_bitcoin_address), 5007);
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
    /// Test that Object timestamp functions are accessible and work correctly
    fun test_object_timestamp_access() {
        // Use the existing setup that creates a DID
        let (creator_signer, _creator_address, creator_public_key_multibase, did_object_id) = 
            did_test_common::setup_did_test_with_creation();
        
        // Test ObjectID-based timestamp access
        let created_timestamp_by_id = did::get_created_timestamp_by_object_id(did_object_id);
        let updated_timestamp_by_id = did::get_updated_timestamp_by_object_id(did_object_id);
        
        // Updated timestamp should be >= created timestamp
        assert!(updated_timestamp_by_id >= created_timestamp_by_id, 15005);
        
        // Clean up compiler warnings
        let _ = creator_signer;
        let _ = creator_public_key_multibase;
    }
} 