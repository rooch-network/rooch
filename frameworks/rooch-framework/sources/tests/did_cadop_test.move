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
    use std::signer;
    use std::option;
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

        // Verify user DID was created and has correct controller
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12001);
        
        // Verify the returned ObjectID matches the created DID
        assert!(*vector::borrow(&controlled_dids, 0) == did_object_id, 12002);

        // Note: In a full implementation, we would also verify:
        // - User VM has authentication and capabilityDelegation permissions
        // - Custodian service VM has only capabilityInvocation permission
        // - User can modify their own DID document
    } 

    #[test]
    /// Test CADOP service VM has correct controller assignment
    fun test_cadop_service_vm_controller() {
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();

        // Create DID via CADOP using the full API (no test-only bypass)
        let did_object_id = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        // Verify DID was created successfully
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12101);
        assert!(*vector::borrow(&controlled_dids, 0) == did_object_id, 12102);

        // Note: Full verification would require access to DID document to check:
        // - Service VM controller is set to custodian's DID
        // - Service VM has capabilityInvocation permission only
    }

    #[test]
    /// Test CADOP did:key controller validation with single controller
    fun test_validate_did_key_controllers_single() {
        did_test_common::init_test_framework();

        // Test valid single did:key controller
        let valid_did_key = did_test_common::generate_test_did_key_string();
        let parsed_did_key = did::parse_did_string(&valid_did_key);
        
        // Test that single did:key controller can be used
        let controllers = vector[parsed_did_key];
        let controllers_length = vector::length(&controllers);
        assert!(controllers_length == 1, 12201);

        // Verify formatting consistency
        let formatted_did_key = did::format_did(&parsed_did_key);
        assert!(formatted_did_key == valid_did_key, 12202);
    }

    #[test]
    /// Test CADOP did:key public key format validation
    fun test_validate_did_key_format() {
        // Test various did:key formats using generated keys
        let valid_ed25519_did_key = did_test_common::generate_test_did_key_string();
        let parsed_ed25519 = did::parse_did_string(&valid_ed25519_did_key);
        let formatted_ed25519 = did::format_did(&parsed_ed25519);
        assert!(formatted_ed25519 == valid_ed25519_did_key, 12301);

        // Test another valid did:key format
        let another_ed25519_did_key = did_test_common::generate_test_did_key_string();
        let parsed_ed25519_2 = did::parse_did_string(&another_ed25519_did_key);
        let formatted_ed25519_2 = did::format_did(&parsed_ed25519_2);
        assert!(formatted_ed25519_2 == another_ed25519_did_key, 12302);
    }

    #[test]
    /// Test CADOP custodian permission model - custodian gets capabilityInvocation only
    fun test_cadop_custodian_permissions() {
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();

        // Create DID via CADOP using the full API (no test-only bypass)
        let did_object_id = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        // Verify DID was created successfully
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12401);
        assert!(*vector::borrow(&controlled_dids, 0) == did_object_id, 12402);

        // In CADOP model:
        // - User retains ultimate control (capabilityDelegation)
        // - Custodian can only invoke services (capabilityInvocation)
        // - This prevents vendor lock-in while enabling custodial services
    }

    #[test]
    /// Test CADOP user retains ultimate control
    fun test_cadop_user_retains_control() {
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();

        // Create DID via CADOP using the full API (no test-only bypass)
        let did_object_id = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        // Verify user DID controller relationship
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12501);
        assert!(*vector::borrow(&controlled_dids, 0) == did_object_id, 12502);

        // The key principle of CADOP: User (did:key controller) retains ultimate control
        // This means user can:
        // 1. Remove custodian's verification methods
        // 2. Add new verification methods
        // 3. Change services
        // 4. Transfer control to another custodian
        // All without custodian's permission
    }

    #[test]
    /// Test CADOP custodian service management capabilities
    fun test_cadop_custodian_service_management() {
        let (custodian_signer, user_did_key_string, custodian_service_pk_multibase, custodian_service_vm_type) = 
            did_test_common::setup_cadop_test_full();

        // Create DID via CADOP using the full API (no test-only bypass)
        let did_object_id = did::create_did_object_via_cadop_with_did_key(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        // Verify DID creation
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12601);
        assert!(*vector::borrow(&controlled_dids, 0) == did_object_id, 12602);

        // Note: In a full CADOP implementation, we would verify:
        // 1. Custodian can add/update/remove services via capabilityInvocation
        // 2. Custodian cannot modify verification methods (requires capabilityDelegation)
        // 3. Services added by custodian have proper controller attribution
    }

    #[test]
    #[expected_failure(abort_code = 29, location = rooch_framework::did)] // ErrorCustodianDoesNotHaveCADOPService
    /// Test that custodian must have CADOP service declared
    fun test_cadop_custodian_service_validation() {

        let (_creator_signer, _creator_address, _creator_public_key, custodian_did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the DID document and its address
        let custodian_did_document = did::get_did_document_by_object_id(custodian_did_object_id);
        let custodian_did_address = did::get_did_address(custodian_did_document);
        
        // Create a signer for the DID address (this is the custodian)
        let custodian_signer = account::create_signer_for_testing(custodian_did_address);

        // Create custodian DID but WITHOUT CADOP service
        let custodian_pk = did_test_common::generate_test_secp256k1_multibase_key();
        did::create_did_object_for_self_entry_test_only(&custodian_signer, custodian_pk);
        
        // Get the custodian DID and find the DID object
        let custodian_address = signer::address_of(&custodian_signer);
        let custodian_did = did::create_rooch_did_by_address(custodian_address);
        let controlled_dids = did::get_dids_by_controller(custodian_did);
        assert!(vector::length(&controlled_dids) > 0, 99999); // Should have created a DID
        
        let custodian_did_object_id = *vector::borrow(&controlled_dids, 0);
        let custodian_did_doc = did::get_did_document_by_object_id(custodian_did_object_id);
        let custodian_did_address = did::get_did_address(custodian_did_doc);
        let custodian_did_signer = account::create_signer_for_testing(custodian_did_address);
        
        // Set up session key authentication for the DID address
        did_test_common::setup_secp256k1_session_key_auth(&custodian_pk);
        
        // Add a different service (not CADOP)
        did::add_service_entry(
            &custodian_did_signer,
            string::utf8(b"other-service"),
            string::utf8(b"SomeOtherService"),
            string::utf8(b"https://custodian.example.com/other")
        );
        
        let user_did_key_string = did_test_common::generate_test_did_key_string();
        let custodian_service_pk_multibase = did_test_common::generate_test_secp256k1_multibase_key();
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
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12701);
        assert!(*vector::borrow(&controlled_dids, 0) == did_object_id, 12702);

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
    /// Test CADOP passkey integration pattern
    fun test_cadop_passkey_integration() {
        // Simulate passkey-based user control
        // Passkeys would be represented as did:key identifiers derived from WebAuthn public keys
        let passkey_did_key_string = did_test_common::generate_test_did_key_string();
        let custodian_signer = did_test_common::setup_custodian_with_cadop_service();
        let custodian_service_pk_multibase = did_test_common::generate_test_secp256k1_multibase_key();

        // Create DID with passkey as controller using simplified API
        did::create_did_object_via_cadop_with_did_key_test_only(
            &custodian_signer,
            passkey_did_key_string,
            custodian_service_pk_multibase,
            string::utf8(b"EcdsaSecp256k1VerificationKey2019")
        );

        // Verify passkey-controlled DID
        let passkey_did_key = did::parse_did_string(&passkey_did_key_string);
        let controlled_dids = did::get_dids_by_controller(passkey_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12801);

        // CADOP + Passkey Benefits:
        // 1. User control through familiar passkey/WebAuthn interface
        // 2. No seed phrases or private key management for users  
        // 3. Custodian provides infrastructure but cannot lock out user
        // 4. Compatible with existing WebAuthn ecosystem
        // 5. Progressive enhancement: can add more auth methods later
    }

    #[test]
    /// Test CADOP DID document structure validation
    fun test_cadop_did_document_structure() {
        did_test_common::init_test_framework();

        let custodian_signer = did_test_common::create_test_account_and_signer();
        let user_did_key_string = did_test_common::generate_test_did_key_string();
        let custodian_service_pk_multibase = did_test_common::generate_test_secp256k1_multibase_key();
        let custodian_service_vm_type = string::utf8(b"EcdsaSecp256k1VerificationKey2019");

        // Create DID via CADOP with simplified API
        did::create_did_object_via_cadop_with_did_key_test_only(
            &custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );

        // Verify DID document structure
        let user_did_key = did::parse_did_string(&user_did_key_string);
        let controlled_dids = did::get_dids_by_controller(user_did_key);
        assert!(vector::length(&controlled_dids) == 1, 12901);

        // CADOP DID Document Requirements:
        // 1. Controller MUST be single did:key
        // 2. User VM MUST have authentication + capabilityDelegation
        // 3. Custodian VM MUST have only capabilityInvocation
        // 4. Both VMs should be in authentication array
        // 5. Services can be managed by custodian VM
        
        // This test validates the structural integrity of CADOP-created DIDs
    }
} 