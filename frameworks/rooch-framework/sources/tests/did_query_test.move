// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Query & Resolution functionality
/// Covers DID existence checks, resolution, controller mapping and queries
module rooch_framework::did_query_test {
    // use rooch_framework::did;
    // use rooch_framework::did_test_common;
    // use std::string;
    // use std::vector;
    // use std::signer;

    // // ========================================
    // // Test Category 5: Query & Resolution Tests
    // // ========================================

    // #[test]
    // /// Test existence check for created DID document
    // fun test_exists_did_document_by_identifier_true() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Test existence by identifier
    //     let did_identifier = moveos_std::address::to_bech32_string(test_address);
    //     assert!(did::exists_did_document_by_identifier(did_identifier), 11001);
    // }

    // #[test]
    // /// Test existence check for non-existent DID document
    // fun test_exists_did_document_by_identifier_false() {
    //     did_test_common::init_test_framework();

    //     // Test non-existent identifier
    //     let nonexistent_identifier = string::utf8(b"bc1qnonexistent123");
    //     assert!(!did::exists_did_document_by_identifier(nonexistent_identifier), 11002);
    // }

    // #[test]
    // /// Test existence check for created DID by address
    // fun test_exists_did_for_address_true() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Test existence by address
    //     assert!(did::exists_did_for_address(test_address), 11003);
    // }

    // #[test]
    // /// Test existence check for non-existent DID by address
    // fun test_exists_did_for_address_false() {
    //     did_test_common::init_test_framework();

    //     // Test non-existent address
    //     let nonexistent_address = @0x999999;
    //     assert!(!did::exists_did_for_address(nonexistent_address), 11004);
    // }

    // #[test]
    // /// Test successful DID resolution by address
    // fun test_get_did_for_address_success() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Get DID by address
    //     let retrieved_did = did::create_rooch_did_by_address(test_address);
    //     let formatted_did = did::format_did(&retrieved_did);
        
    //     // Verify correct DID was retrieved
    //     let expected_did_string = string::utf8(b"did:rooch:");
    //     let test_bech32 = moveos_std::address::to_bech32_string(test_address);
    //     string::append(&mut expected_did_string, test_bech32);
    //     assert!(formatted_did == expected_did_string, 11005);
    // }

    // #[test]
    // /// Test getting DIDs controlled by a specific controller
    // fun test_get_dids_by_controller_single() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Get DIDs controlled by this address
    //     let controller_did = did::create_rooch_did_by_address(test_address);
    //     let controlled_dids = did::get_dids_by_controller(controller_did);
        
    //     // Should have exactly one DID
    //     assert!(vector::length(&controlled_dids) == 1, 11101);
    // }

    // #[test]
    // /// Test getting DIDs for controller with no DIDs
    // fun test_get_dids_by_controller_empty() {
    //     did_test_common::init_test_framework();

    //     // Create a controller DID that doesn't control any DIDs
    //     let controller_did = did::create_did_from_parts(
    //         string::utf8(b"rooch"),
    //         string::utf8(b"bc1qnonexistent123")
    //     );
    //     let controlled_dids = did::get_dids_by_controller(controller_did);
        
    //     // Should have no DIDs
    //     assert!(vector::length(&controlled_dids) == 0, 11102);
    // }

    // #[test]
    // /// Test getting DIDs by controller string
    // fun test_get_dids_by_controller_string() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Get DIDs controlled by this address using string format
    //     let controller_did_string = string::utf8(b"did:rooch:");
    //     let test_bech32 = moveos_std::address::to_bech32_string(test_address);
    //     string::append(&mut controller_did_string, test_bech32);
        
    //     let controlled_dids = did::get_dids_by_controller_string(controller_did_string);
        
    //     // Should have exactly one DID
    //     assert!(vector::length(&controlled_dids) == 1, 11103);
    // }

    // #[test]
    // /// Test multiple DIDs controlled by same controller
    // fun test_get_dids_by_controller_multiple() {
    //     // Note: In the current implementation, each address can only have one DID
    //     // This test demonstrates the expected behavior if multiple DIDs were allowed
    //     did_test_common::init_test_framework();

    //     let test_signer1 = did_test_common::create_test_account_and_signer();
    //     let test_address1 = signer::address_of(&test_signer1);
    //     let test_public_key1 = did_test_common::generate_test_secp256k1_multibase_key();

    //     let test_signer2 = moveos_std::account::create_signer_for_testing(@0x999);
    //     let test_address2 = signer::address_of(&test_signer2);
    //     let test_public_key2 = string::utf8(b"z21pGXTKbEq9G4f4z8qNFXSZvSiQ8B1X3i9Y5v7xK2m1n5");

    //     // Create first DID
    //     did::create_did_object_for_self_entry_test_only(&test_signer1, test_public_key1);

    //     // Create second DID  
    //     did::create_did_object_for_self_entry_test_only(&test_signer2, test_public_key2);

    //     // Each should control exactly one DID (themselves)
    //     let controller_did1 = did::create_rooch_did_by_address(test_address1);
    //     let controlled_dids1 = did::get_dids_by_controller(controller_did1);
    //     assert!(vector::length(&controlled_dids1) == 1, 11201);

    //     let controller_did2 = did::create_rooch_did_by_address(test_address2);
    //     let controlled_dids2 = did::get_dids_by_controller(controller_did2);
    //     assert!(vector::length(&controlled_dids2) == 1, 11202);
    // }

    // #[test]
    // /// Test DID resolution with deterministic ObjectID generation
    // fun test_resolve_did_object_id_deterministic() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Verify DID can be resolved
    //     assert!(did::exists_did_for_address(test_address), 11301);

    //     // Test that the same address always resolves to the same DID
    //     let did1 = did::create_rooch_did_by_address(test_address);
    //     let did2 = did::create_rooch_did_by_address(test_address);
        
    //     let formatted_did1 = did::format_did(&did1);
    //     let formatted_did2 = did::format_did(&did2);
        
    //     assert!(formatted_did1 == formatted_did2, 11302);
    // }

    // #[test]
    // /// Test querying DID with specific identifier format
    // fun test_did_identifier_resolution() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Test resolution by bech32 identifier
    //     let bech32_identifier = moveos_std::address::to_bech32_string(test_address);
    //     assert!(did::exists_did_document_by_identifier(bech32_identifier), 11401);

    //     // Verify DID document can be retrieved by identifier
    //     let retrieved_did = did::create_rooch_did_by_address(test_address);
    //     let expected_did_string = string::utf8(b"did:rooch:");
    //     string::append(&mut expected_did_string, bech32_identifier);
    //     let formatted_retrieved = did::format_did(&retrieved_did);
        
    //     assert!(formatted_retrieved == expected_did_string, 11402);
    // }

    // #[test]
    // /// Test controller mapping with did:key controllers
    // fun test_controller_mapping_did_key() {
    //     did_test_common::init_test_framework();

    //     // Test did:key controller creation
    //     let did_key_string = string::utf8(b"did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
    //     let did_key = did::parse_did_string(&did_key_string);
        
    //     // Test formatting and parsing consistency
    //     let formatted_did_key = did::format_did(&did_key);
    //     assert!(formatted_did_key == did_key_string, 11501);

    //     // Test controller queries with did:key (should be empty initially)
    //     let controlled_dids = did::get_dids_by_controller(did_key);
    //     assert!(vector::length(&controlled_dids) == 0, 11502);
    // }

    // #[test]
    // /// Test controller mapping with rooch DIDs
    // fun test_controller_mapping_rooch_did() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Test rooch DID controller
    //     let rooch_controller = did::create_rooch_did_by_address(test_address);
    //     let controlled_dids = did::get_dids_by_controller(rooch_controller);
        
    //     // Should control exactly one DID (itself)
    //     assert!(vector::length(&controlled_dids) == 1, 11601);
        
    //     // Verify the controlled DID matches expectation
    //     let controlled_did = *vector::borrow(&controlled_dids, 0);
    //     // Note: controlled_did is ObjectID, we need to get the actual DID document
    //     let controlled_did_doc = did::get_did_document_by_object_id(controlled_did);
    //     let controlled_did_identifier = did::get_did_identifier(controlled_did_doc);
    //     let formatted_controlled = did::format_did(controlled_did_identifier);
    //     let formatted_controller = did::format_did(&rooch_controller);
        
    //     // In self-controlled scenario, controller and controlled are the same
    //     assert!(formatted_controlled == formatted_controller, 11602);
    // }

    // #[test]
    // /// Test edge cases in DID queries
    // fun test_did_query_edge_cases() {
    //     did_test_common::init_test_framework();

    //     // Test with empty DID registry
    //     let nonexistent_controller = did::create_did_from_parts(
    //         string::utf8(b"rooch"), 
    //         string::utf8(b"bc1qneverexisted")
    //     );
    //     let empty_result = did::get_dids_by_controller(nonexistent_controller);
    //     assert!(vector::length(&empty_result) == 0, 11701);

    //     // Test existence check with various invalid identifiers
    //     assert!(!did::exists_did_document_by_identifier(string::utf8(b"invalid")), 11702);
    //     assert!(!did::exists_did_document_by_identifier(string::utf8(b"")), 11703);
    //     assert!(!did::exists_did_document_by_identifier(string::utf8(b"bc1q")), 11704);

    //     // Test address-based existence with special addresses
    //     assert!(!did::exists_did_for_address(@0x0), 11705);
    //     assert!(!did::exists_did_for_address(@0x1), 11706);
    //     assert!(!did::exists_did_for_address(@0xffffffffffffffffffffffffffffffff), 11707);
    // }

    // #[test]
    // /// Test query performance with multiple DIDs
    // fun test_query_multiple_dids_performance() {
    //     did_test_common::init_test_framework();

    //     let dids_count = 5;
    //     let signers = vector::empty<signer>();
    //     let addresses = vector::empty<address>();
        
    //     // Create multiple DIDs
    //     let i = 0;
    //     while (i < dids_count) {
    //         let test_address = if (i == 0) { @0x100 }
    //                           else if (i == 1) { @0x200 }
    //                           else if (i == 2) { @0x300 }
    //                           else if (i == 3) { @0x400 }
    //                           else { @0x500 };
            
    //         let test_signer = moveos_std::account::create_signer_for_testing(test_address);
    //         let test_public_key = if (i == 0) { string::utf8(b"z21pGXTKbEq9G4f4z8qNFXSZvSiQ8B1X3i9Y5v7xK2m1n1") }
    //                              else if (i == 1) { string::utf8(b"z21pGXTKbEq9G4f4z8qNFXSZvSiQ8B1X3i9Y5v7xK2m1n2") }
    //                              else if (i == 2) { string::utf8(b"z21pGXTKbEq9G4f4z8qNFXSZvSiQ8B1X3i9Y5v7xK2m1n3") }
    //                              else if (i == 3) { string::utf8(b"z21pGXTKbEq9G4f4z8qNFXSZvSiQ8B1X3i9Y5v7xK2m1n4") }
    //                              else { string::utf8(b"z21pGXTKbEq9G4f4z8qNFXSZvSiQ8B1X3i9Y5v7xK2m1n5") };

    //         did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key);
            
    //         vector::push_back(&mut signers, test_signer);
    //         vector::push_back(&mut addresses, test_address);
    //         i = i + 1;
    //     };

    //     // Test that all DIDs can be queried
    //     let j = 0;
    //     while (j < dids_count) {
    //         let address = *vector::borrow(&addresses, j);
    //         assert!(did::exists_did_for_address(address), 11800 + j);
            
    //         // Test controller mapping
    //         let controller_did = did::create_rooch_did_by_address(address);
    //         let controlled_dids = did::get_dids_by_controller(controller_did);
    //         assert!(vector::length(&controlled_dids) == 1, 11810 + j);
            
    //         j = j + 1;
    //     };

    //     // Clean up compiler warnings
    //     let _ = signers;
    // }

    // #[test]
    // /// Test DID resolution consistency across different query methods
    // fun test_resolution_consistency() {
    //     did_test_common::init_test_framework();

    //     let test_signer = did_test_common::create_test_account_and_signer();
    //     let test_address = signer::address_of(&test_signer);
    //     let test_public_key_multibase = did_test_common::generate_test_secp256k1_multibase_key();

    //     // Create DID first
    //     did::create_did_object_for_self_entry_test_only(&test_signer, test_public_key_multibase);

    //     // Method 1: Query by address existence
    //     assert!(did::exists_did_for_address(test_address), 11901);

    //     // Method 2: Query by identifier existence
    //     let identifier = moveos_std::address::to_bech32_string(test_address);
    //     assert!(did::exists_did_document_by_identifier(identifier), 11902);

    //     // Method 3: Create DID by address and verify format
    //     let did_by_address = did::create_rooch_did_by_address(test_address);
    //     let expected_did_string = string::utf8(b"did:rooch:");
    //     string::append(&mut expected_did_string, identifier);
    //     let formatted_did = did::format_did(&did_by_address);
    //     assert!(formatted_did == expected_did_string, 11903);

    //     // Method 4: Controller mapping consistency
    //     let controlled_dids = did::get_dids_by_controller(did_by_address);
    //     assert!(vector::length(&controlled_dids) == 1, 11904);
        
    //     let controlled_did = *vector::borrow(&controlled_dids, 0);
    //     // Note: controlled_did is ObjectID, we need to get the actual DID document  
    //     let controlled_did_doc = did::get_did_document_by_object_id(controlled_did);
    //     let controlled_did_identifier = did::get_did_identifier(controlled_did_doc);
    //     let formatted_controlled = did::format_did(controlled_did_identifier);
    //     assert!(formatted_controlled == formatted_did, 11905);
    // }
} 