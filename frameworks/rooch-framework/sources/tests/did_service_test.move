// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// Tests for DID Service Management functionality
/// Covers adding, updating, removing services and property management
module rooch_framework::did_service_test {
    use rooch_framework::did;
    use rooch_framework::did_test_common;
    use std::string;
    use std::vector;
    use moveos_std::account;

    // ========================================
    // Test Category 3: Service Management Tests  
    // ========================================

    #[test]
    /// Test adding service with custom properties successfully
    fun test_add_service_with_properties() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();
        
        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        
        // Create a signer for the DID address
        let did_signer = account::create_signer_for_testing(did_address);

        // Add service with properties using the correct DID signer
        let fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");
        let property_keys = vector[
            string::utf8(b"version"),
            string::utf8(b"protocol"),
            string::utf8(b"authentication")
        ];
        let property_values = vector[
            string::utf8(b"1.0"),
            string::utf8(b"HTTPS"),
            string::utf8(b"Bearer")
        ];

        did::add_service_with_properties_entry(
            &did_signer,
            fragment,
            service_type,
            service_endpoint,
            property_keys,
            property_values
        );

        // Verify service was added using the correct DID address
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_after, &fragment), 9101);
    }

    #[test]
    #[expected_failure(abort_code = 7, location = rooch_framework::did)] // ErrorServiceAlreadyExists
    /// Test adding service with duplicate fragment fails
    fun test_add_service_already_exists() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add service first time
        let fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");

        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);

        // Try to add service with same fragment - should fail
        let service_type2 = string::utf8(b"AnotherService");
        let service_endpoint2 = string::utf8(b"https://another.example.com");
        did::add_service_entry(&did_signer, fragment, service_type2, service_endpoint2);
    }

    #[test]
    #[expected_failure(abort_code = 16, location = rooch_framework::did)] // ErrorPropertyKeysValuesLengthMismatch
    /// Test adding service with mismatched property arrays fails
    fun test_add_service_property_length_mismatch() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Try to add service with mismatched property arrays
        let fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");
        let property_keys = vector[string::utf8(b"key1"), string::utf8(b"key2")]; // 2 keys
        let property_values = vector[string::utf8(b"value1")]; // 1 value - mismatch!

        did::add_service_with_properties_entry(
            &did_signer,
            fragment,
            service_type,
            service_endpoint,
            property_keys,
            property_values
        );
    }

    #[test]
    /// Test updating service successfully
    fun test_update_service_success() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add service first
        let fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");

        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);

        // Verify service exists
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_check, &fragment), 9201);

        // Update service
        let new_service_type = string::utf8(b"UpdatedTestService");
        let new_service_endpoint = string::utf8(b"https://updated.example.com");
        let new_property_keys = vector[string::utf8(b"version")];
        let new_property_values = vector[string::utf8(b"2.0")];

        did::update_service_entry(
            &did_signer,
            fragment,
            new_service_type,
            new_service_endpoint,
            new_property_keys,
            new_property_values
        );

        // Verify service still exists (update doesn't remove it)
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_after, &fragment), 9202);
    }

    #[test]
    #[expected_failure(abort_code = 6, location = rooch_framework::did)] // ErrorServiceNotFound
    /// Test updating non-existent service fails
    fun test_update_service_not_found() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Try to update non-existent service
        let fragment = string::utf8(b"nonexistent-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");
        let property_keys = vector::empty<string::String>();
        let property_values = vector::empty<string::String>();

        did::update_service_entry(
            &did_signer,
            fragment,
            service_type,
            service_endpoint,
            property_keys,
            property_values
        );
    }

    #[test]
    /// Test removing service successfully
    fun test_remove_service_success() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add service first
        let fragment = string::utf8(b"test-service");
        let service_type = string::utf8(b"TestService");
        let service_endpoint = string::utf8(b"https://test.example.com");

        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);

        // Verify service exists
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_check, &fragment), 9301);

        // Remove service
        did::remove_service_entry(&did_signer, fragment);

        // Verify service was removed
        let did_document_after = did::get_did_document_by_address(did_address);
        assert!(!did::test_service_exists(did_document_after, &fragment), 9302);
    }

    #[test]
    #[expected_failure(abort_code = 6, location = rooch_framework::did)] // ErrorServiceNotFound
    /// Test removing non-existent service fails
    fun test_remove_service_not_found() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Try to remove non-existent service
        let nonexistent_fragment = string::utf8(b"nonexistent-service");
        did::remove_service_entry(&did_signer, nonexistent_fragment);
    }

    #[test]
    /// Test adding multiple services to same DID
    fun test_multiple_services_management() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add multiple services
        let service_count = 3;
        let i = 0;
        while (i < service_count) {
            let service_fragment = string::utf8(b"service-");
            let index_str = if (i == 0) { string::utf8(b"0") }
                           else if (i == 1) { string::utf8(b"1") }
                           else { string::utf8(b"2") };
            string::append(&mut service_fragment, index_str);
            
            let service_type = string::utf8(b"TestService");
            let service_endpoint = string::utf8(b"https://service-");
            string::append(&mut service_endpoint, index_str);
            string::append_utf8(&mut service_endpoint, b".example.com");

            did::add_service_entry(&did_signer, service_fragment, service_type, service_endpoint);
            i = i + 1;
        };

        // Verify all services exist
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_check, &string::utf8(b"service-0")), 9401);
        assert!(did::test_service_exists(did_document_check, &string::utf8(b"service-1")), 9402);
        assert!(did::test_service_exists(did_document_check, &string::utf8(b"service-2")), 9403);
    }

    #[test]
    /// Test service with empty properties
    fun test_add_service_empty_properties() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add service with empty properties
        let fragment = string::utf8(b"simple-service");
        let service_type = string::utf8(b"SimpleService");
        let service_endpoint = string::utf8(b"https://simple.example.com");
        let empty_keys = vector::empty<string::String>();
        let empty_values = vector::empty<string::String>();

        did::add_service_with_properties_entry(
            &did_signer,
            fragment,
            service_type,
            service_endpoint,
            empty_keys,
            empty_values
        );

        // Verify service was added
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_check, &fragment), 9501);
    }

    #[test]
    /// Test service update with property changes
    fun test_service_property_updates() {
        let (_test_signer, _creator_address, _initial_public_key, did_object_id) = did_test_common::setup_did_test_with_creation();

        // Get the actual DID document and its address
        let did_document = did::get_did_document_by_object_id(did_object_id);
        let did_address = did::get_did_address(did_document);
        let did_signer = account::create_signer_for_testing(did_address);

        // Add service with initial properties
        let fragment = string::utf8(b"versioned-service");
        let service_type = string::utf8(b"VersionedService");
        let service_endpoint = string::utf8(b"https://v1.example.com");
        let initial_keys = vector[string::utf8(b"version")];
        let initial_values = vector[string::utf8(b"1.0")];

        did::add_service_with_properties_entry(
            &did_signer,
            fragment,
            service_type,
            service_endpoint,
            initial_keys,
            initial_values
        );

        // Update service with new properties
        let new_service_type = string::utf8(b"VersionedServiceV2");
        let new_service_endpoint = string::utf8(b"https://v2.example.com");
        let updated_keys = vector[
            string::utf8(b"version"),
            string::utf8(b"deprecation_date")
        ];
        let updated_values = vector[
            string::utf8(b"2.0"),
            string::utf8(b"2024-12-31")
        ];

        did::update_service_entry(
            &did_signer,
            fragment,
            new_service_type,
            new_service_endpoint,
            updated_keys,
            updated_values
        );

        // Verify service still exists after update
        let did_document_check = did::get_did_document_by_address(did_address);
        assert!(did::test_service_exists(did_document_check, &fragment), 9601);
    }
} 