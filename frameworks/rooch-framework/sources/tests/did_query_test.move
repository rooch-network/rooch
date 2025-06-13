// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Query & Resolution functionality
/// Covers DID existence checks, resolution, controller mapping and queries
module rooch_framework::did_query_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::vector;

    // ========================================
    // Test Category 5: Query & Resolution Tests
    // ========================================

    #[test]
    /// Test existence check for created DID document
    fun test_exists_did_document_by_identifier_true() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Test existence by identifier
        let did_identifier = moveos_std::address::to_bech32_string(did_address);
        assert!(did::exists_did_document_by_identifier(did_identifier), 11001);
    }

    #[test]
    /// Test existence check for non-existent DID document
    fun test_exists_did_document_by_identifier_false() {
        did_test_common::init_test_framework();

        // Test non-existent identifier
        let nonexistent_identifier = string::utf8(b"bc1qnonexistent123");
        assert!(!did::exists_did_document_by_identifier(nonexistent_identifier), 11002);
    }

    #[test]
    /// Test existence check for created DID by address
    fun test_exists_did_for_address_true() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Test existence by address
        assert!(did::exists_did_for_address(did_address), 11003);
    }

    #[test]
    /// Test existence check for non-existent DID by address
    fun test_exists_did_for_address_false() {
        did_test_common::init_test_framework();

        // Test non-existent address
        let nonexistent_address = @0x999999;
        assert!(!did::exists_did_for_address(nonexistent_address), 11004);
    }

    #[test]
    /// Test successful DID resolution by address
    fun test_get_did_for_address_success() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Get DID by address
        let retrieved_did = did::new_rooch_did_by_address(did_address);
        let formatted_did = did::format_did(&retrieved_did);
        
        // Verify correct DID was retrieved
        let expected_did_string = string::utf8(b"did:rooch:");
        let test_bech32 = moveos_std::address::to_bech32_string(did_address);
        string::append(&mut expected_did_string, test_bech32);
        assert!(formatted_did == expected_did_string, 11005);
    }

    #[test]
    /// Test getting DIDs controlled by a specific controller
    fun test_get_dids_by_controller_single() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let _did_address = did::get_did_address(did_document);

        // Get DIDs controlled by the creator address (not the DID address)
        let controller_did = did::new_rooch_did_by_address(creator_address);
        let controlled_dids = did::get_dids_by_controller(controller_did);
        
        // Should have exactly one DID
        assert!(vector::length(&controlled_dids) == 1, 11101);
    }

    #[test]
    /// Test getting DIDs for controller with no DIDs
    fun test_get_dids_by_controller_empty() {
        did_test_common::init_test_framework();

        // Create a controller DID that doesn't control any DIDs
        let controller_did = did::new_did_from_parts(
            string::utf8(b"rooch"),
            string::utf8(b"bc1qnonexistent123")
        );
        let controlled_dids = did::get_dids_by_controller(controller_did);
        
        // Should have no DIDs
        assert!(vector::length(&controlled_dids) == 0, 11102);
    }

    #[test]
    /// Test getting DIDs by controller string
    fun test_get_dids_by_controller_string() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let _did_address = did::get_did_address(did_document);

        // Get DIDs controlled by the creator address using string format
        let controller_did_string = string::utf8(b"did:rooch:");
        let creator_bech32 = moveos_std::address::to_bech32_string(creator_address);
        string::append(&mut controller_did_string, creator_bech32);
        
        let controlled_dids = did::get_dids_by_controller_string(controller_did_string);
        
        // Should have exactly one DID
        assert!(vector::length(&controlled_dids) == 1, 11103);
    } 

    #[test]
    /// Test DID resolution with deterministic ObjectID generation
    fun test_resolve_did_object_id_deterministic() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Verify DID can be resolved
        assert!(did::exists_did_for_address(did_address), 11301);

        // Test that the same address always resolves to the same DID
        let did1 = did::new_rooch_did_by_address(did_address);
        let did2 = did::new_rooch_did_by_address(did_address);
        
        let formatted_did1 = did::format_did(&did1);
        let formatted_did2 = did::format_did(&did2);
        
        assert!(formatted_did1 == formatted_did2, 11302);
    }

    #[test]
    /// Test querying DID with specific identifier format
    fun test_did_identifier_resolution() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, _creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Test resolution by bech32 identifier
        let bech32_identifier = moveos_std::address::to_bech32_string(did_address);
        assert!(did::exists_did_document_by_identifier(bech32_identifier), 11401);

        // Verify DID document can be retrieved by identifier
        let retrieved_did = did::new_rooch_did_by_address(did_address);
        let expected_did_string = string::utf8(b"did:rooch:");
        string::append(&mut expected_did_string, bech32_identifier);
        let formatted_retrieved = did::format_did(&retrieved_did);
        
        assert!(formatted_retrieved == expected_did_string, 11402);
    }

    #[test]
    /// Test controller mapping with did:key controllers
    fun test_controller_mapping_did_key() {
        did_test_common::init_test_framework();

        // Test did:key controller creation
        let did_key_string = string::utf8(b"did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
        let did_key = did::parse_did_string(&did_key_string);
        
        // Test formatting and parsing consistency
        let formatted_did_key = did::format_did(&did_key);
        assert!(formatted_did_key == did_key_string, 11501);

        // Test controller queries with did:key (should be empty initially)
        let controlled_dids = did::get_dids_by_controller(did_key);
        assert!(vector::length(&controlled_dids) == 0, 11502);
    }

    #[test]
    /// Test controller mapping with rooch DIDs
    fun test_controller_mapping_rooch_did() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Test rooch DID controller - use creator address as controller
        let rooch_controller = did::new_rooch_did_by_address(creator_address);
        let controlled_dids = did::get_dids_by_controller(rooch_controller);
        
        // Should control exactly one DID (the created DID)
        assert!(vector::length(&controlled_dids) == 1, 11601);
        
        // Verify the controlled DID matches expectation
        let controlled_did = *vector::borrow(&controlled_dids, 0);
        // Note: controlled_did is ObjectID, we need to get the actual DID document
        let controlled_did_doc = did::get_did_document(controlled_did);
        let controlled_did_identifier = did::doc_id(controlled_did_doc);
        let formatted_controlled = did::format_did(controlled_did_identifier);
        
        // The controlled DID should be the DID at did_address
        let expected_controlled_did = did::new_rooch_did_by_address(did_address);
        let formatted_expected = did::format_did(&expected_controlled_did);
        
        assert!(formatted_controlled == formatted_expected, 11602);
    }

    #[test]
    /// Test edge cases in DID queries
    fun test_did_query_edge_cases() {
        did_test_common::init_test_framework();

        // Test with empty DID registry
        let nonexistent_controller = did::new_did_from_parts(
            string::utf8(b"rooch"), 
            string::utf8(b"bc1qneverexisted")
        );
        let empty_result = did::get_dids_by_controller(nonexistent_controller);
        assert!(vector::length(&empty_result) == 0, 11701);

        // Test existence check with various invalid identifiers
        assert!(!did::exists_did_document_by_identifier(string::utf8(b"invalid")), 11702);
        assert!(!did::exists_did_document_by_identifier(string::utf8(b"")), 11703);
        assert!(!did::exists_did_document_by_identifier(string::utf8(b"bc1q")), 11704);

        // Test address-based existence with special addresses
        assert!(!did::exists_did_for_address(@0x0), 11705);
        assert!(!did::exists_did_for_address(@0x1), 11706);
        assert!(!did::exists_did_for_address(@0xffffffffffffffffffffffffffffffff), 11707);
    }

    #[test]
    /// Test query performance with DID
    fun test_query_multiple_dids_performance() {
        // Create DID using proper setup
        let (_creator_signer, creator_address, _creator_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        
        // Test DID existence
        assert!(did::exists_did_for_address(did_address), 11800);
        
        // Test controller mapping
        let controller_did = did::new_rooch_did_by_address(creator_address);
        let controlled_dids = did::get_dids_by_controller(controller_did);
        assert!(vector::length(&controlled_dids) == 1, 11810);
        
        // Test DID identifier resolution
        let identifier = moveos_std::address::to_bech32_string(did_address);
        assert!(did::exists_did_document_by_identifier(identifier), 11801);
        
        // Test DID creation by address
        let did_by_address = did::new_rooch_did_by_address(did_address);
        let expected_did_string = string::utf8(b"did:rooch:");
        string::append(&mut expected_did_string, identifier);
        let formatted_did = did::format_did(&did_by_address);
        assert!(formatted_did == expected_did_string, 11802);
    }

    #[test]
    /// Test DID resolution consistency across different query methods
    fun test_resolution_consistency() {
        // Use proper setup to get creator info and DID object ID
        let (_creator_signer, creator_address, _creator_public_key_multibase, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);

        // Method 1: Query by address existence
        assert!(did::exists_did_for_address(did_address), 11901);

        // Method 2: Query by identifier existence
        let identifier = moveos_std::address::to_bech32_string(did_address);
        assert!(did::exists_did_document_by_identifier(identifier), 11902);

        // Method 3: Create DID by address and verify format
        let did_by_address = did::new_rooch_did_by_address(did_address);
        let expected_did_string = string::utf8(b"did:rooch:");
        string::append(&mut expected_did_string, identifier);
        let formatted_did = did::format_did(&did_by_address);
        assert!(formatted_did == expected_did_string, 11903);

        // Method 4: Controller mapping consistency - use creator address as controller
        let controller_did = did::new_rooch_did_by_address(creator_address);
        let controlled_dids = did::get_dids_by_controller(controller_did);
        assert!(vector::length(&controlled_dids) == 1, 11904);
        
        let controlled_did = *vector::borrow(&controlled_dids, 0);
        // Note: controlled_did is ObjectID, we need to get the actual DID document  
        let controlled_did_doc = did::get_did_document(controlled_did);
        let controlled_did_identifier = did::doc_id(controlled_did_doc);
        let formatted_controlled = did::format_did(controlled_did_identifier);
        assert!(formatted_controlled == formatted_did, 11905);
    }
} 