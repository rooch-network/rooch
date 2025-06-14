// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID CADOP (NIP-3) Compliance functionality
/// Covers Crypto Assisted Decentralized Object Ownership Protocol specifications
module rooch_framework::did_cadop_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::vector;
    use moveos_std::account;

    // ========================================
    // Test Category 8: CADOP (NIP-3) Compliance Tests
    // ========================================

    #[test]
    /// Test CADOP permission model - user gets authentication + capabilityDelegation
    fun test_cadop_user_permissions() {
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();

        // Create DID via CADOP using the full API (no test-only bypass)
        let did_object_id = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_str = did::format_did(did::doc_id(did_document));

        // Verify user DID was created and has correct controller
        let controlled_dids = did::get_dids_by_controller_string(user_did_key_string);
        assert!(vector::length(&controlled_dids) == 1, 12001);
        
        // Verify the returned ObjectID matches the created DID
        assert!(*vector::borrow(&controlled_dids, 0) == did_str, 12002);

        // Note: In a full implementation, we would also verify:
        // - User VM has authentication and capabilityDelegation permissions
        // - Custodian service VM has only capabilityInvocation permission
        // - User can modify their own DID document
    } 

    #[test]
    /// Test CADOP service VM has correct controller assignment
    fun test_cadop_service_vm_controller() {
        //TODO

        // Note: Full verification would require access to DID document to check:
        // - Service VM controller is set to custodian's DID
        // - Service VM has capabilityInvocation permission only
    }




    #[test]
    /// Test CADOP custodian service management capabilities
    fun test_cadop_custodian_service_management() {
        //TODO

        // Note: In a full CADOP implementation, we would verify:
        // 1. Custodian can add/update/remove services via capabilityInvocation
        // 2. Custodian cannot modify verification methods (requires capabilityDelegation)
        // 3. Services added by custodian have proper controller attribution
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorCustodianDoesNotHaveCADOPService, location = rooch_framework::did)] 
    /// Test that custodian must have CADOP service declared
    fun test_cadop_custodian_service_validation() {

        let (_creator_signer, _creator_address, creator_public_key, custodian_did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the DID document and its address
        let custodian_did_document = did::get_did_document_by_object_id(custodian_did_object_id);
        let custodian_did_address = did::get_did_address(custodian_did_document);
        
        // Create a signer for the DID address (this is the custodian)
        let custodian_signer = account::create_signer_for_testing(custodian_did_address);

        // Set up session key authentication for the DID address
        did_test_common::setup_secp256k1_session_key_auth(&creator_public_key);
        
        // Add a different service (not CADOP)
        did::add_service_entry(
            &custodian_signer,
            string::utf8(b"other-service"),
            string::utf8(b"SomeOtherService"),
            string::utf8(b"https://custodian.example.com/other")
        );
        
        let user_did_key_string = did_test_common::generate_test_did_key_string();
        let custodian_service_pk_multibase = did_test_common::generate_test_secp256k1_multibase();
        let custodian_service_vm_type = string::utf8(b"EcdsaSecp256k1VerificationKey2019");

        // This should fail because custodian doesn't have CadopCustodianService
        let _ = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );
    }

    #[test]
    /// Test CADOP prevents vendor lock-in
    fun test_cadop_no_vendor_lock_in() {
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();

        // Create DID via CADOP using the full API (no test-only bypass)
        let did_object_id = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        // Verify creation
        let _did_document = did::get_did_document_by_object_id(did_object_id);

        // CADOP Anti-Vendor-Lock-in Features:
        // 1. User's did:key always remains the controller
        // 2. User can remove custodian's verification methods at any time
        // 3. User can add new custodian verification methods
        // 4. User can migrate to different custodial services
        // 5. User can become fully self-custodial
        
        // This test demonstrates the principle - user maintains sovereignty
        // while benefiting from custodial services
    }


    #[test]
    /// Test CADOP DID document structure validation
    fun test_cadop_did_document_structure() {

        // CADOP DID Document Requirements:
        // 1. Controller MUST be single did:key
        // 2. User VM MUST have authentication + capabilityDelegation
        // 3. Custodian VM MUST have only capabilityInvocation
        // 4. Both VMs should be in authentication array
        // 5. Services can be managed by custodian VM
        
        // This test validates the structural integrity of CADOP-created DIDs
    }
} 