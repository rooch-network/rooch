// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::did_limits_test {
    use std::vector;
    use std::string;
    use moveos_std::account;
    use rooch_framework::did;
    use rooch_framework::did_test_common;

    /// Helper function to convert u64 to string for unique fragment generation
    fun u64_to_string(n: u64): string::String {
        if (n == 0) {
            return string::utf8(b"0")
        };

        let result = string::utf8(b"");
        let num = n;
        while (num > 0) {
            let digit = num % 10;
            let char = if (digit == 0) { 48 }
                       else if (digit == 1) { 49 }
                       else if (digit == 2) { 50 }
                       else if (digit == 3) { 51 }
                       else if (digit == 4) { 52 }
                       else if (digit == 5) { 53 }
                       else if (digit == 6) { 54 }
                       else if (digit == 7) { 55 }
                       else if (digit == 8) { 56 }
                       else { 57 }; // digit == 9
            string::insert(&mut result, 0, string::utf8(vector[char]));
            num = num / 10;
        };
        result
    }

    // Ensure create path works (no limit checks at creation time) and
    // document-level VM cap is enforced when adding more methods post-create.
    #[test]
    fun test_create_then_add_methods_until_doc_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Add 63 additional verification methods (doc initially has 1 account-key)
        let i = 0u64;
        while (i < 63) {
            let key = did_test_common::generate_test_secp256k1_multibase();
            let fragment = key; // use key string as fragment to ensure uniqueness
            let method_type = did::verification_method_type_secp256k1();
            let empty_rels = vector::empty<u8>();
            did::add_verification_method_entry(&did_signer, fragment, method_type, key, empty_rels);
            i = i + 1;
        };
        // Reaching here means creation succeeded and 63 adds succeeded (total 64 VMs in doc)
    }

    // The 64th additional add (making total 65) should fail with document limit error
    #[test]
    #[expected_failure(abort_code = did::ErrorTooManyVerificationMethods, location = did)]
    fun test_add_verification_methods_exceeds_doc_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Fill to the doc limit (64 total): add 63 additional methods
        let i = 0u64;
        while (i < 63) {
            let key = did_test_common::generate_test_secp256k1_multibase();
            let fragment = key;
            let method_type = did::verification_method_type_secp256k1();
            let empty_rels = vector::empty<u8>();
            did::add_verification_method_entry(&did_signer, fragment, method_type, key, empty_rels);
            i = i + 1;
        };

        // Attempt to add one more should abort due to document-level cap
        let key = did_test_common::generate_test_secp256k1_multibase();
        let fragment = key;
        let method_type = did::verification_method_type_secp256k1();
        let empty_rels = vector::empty<u8>();
        did::add_verification_method_entry(&did_signer, fragment, method_type, key, empty_rels);
    }

    // Test services limit: add 32 services should work, 33rd should fail
    #[test]
    fun test_add_services_until_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Add 32 services (should succeed)
        let i = 0u64;
        while (i < 32) {
            let fragment = u64_to_string(i);
            let service_type = std::string::utf8(b"TestService");
            let service_endpoint = std::string::utf8(b"https://example.com");
            did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);
            i = i + 1;
        };
        // Reaching here means all 32 services were added successfully
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorTooManyServices, location = did)]
    fun test_add_services_exceeds_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Add 32 services (should succeed)
        let i = 0u64;
        while (i < 32) {
            let fragment = u64_to_string(i);
            let service_type = std::string::utf8(b"TestService");
            let service_endpoint = std::string::utf8(b"https://example.com");
            did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);
            i = i + 1;
        };

        // 33rd service should fail
        let fragment = u64_to_string(32);
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");
        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);
    }

    // Test service properties limit: 16 properties should work, 17th should fail
    #[test]
    fun test_add_service_with_max_properties() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");

        // Create 16 properties
        let property_keys = vector::empty<std::string::String>();
        let property_values = vector::empty<std::string::String>();
        let i = 0u64;
        while (i < 16) {
            let key = std::string::utf8(b"key");
            std::string::append(&mut key, u64_to_string(i));
            let value = std::string::utf8(b"value");
            std::string::append(&mut value, u64_to_string(i));
            vector::push_back(&mut property_keys, key);
            vector::push_back(&mut property_values, value);
            i = i + 1;
        };

        did::add_service_with_properties_entry(&did_signer, fragment, service_type, service_endpoint, property_keys, property_values);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorTooManyServiceProperties, location = did)]
    fun test_add_service_exceeds_properties_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");

        // Create 17 properties (exceeds limit)
        let property_keys = vector::empty<std::string::String>();
        let property_values = vector::empty<std::string::String>();
        let i = 0u64;
        while (i < 17) {
            let key = std::string::utf8(b"key");
            std::string::append(&mut key, u64_to_string(i));
            let value = std::string::utf8(b"value");
            std::string::append(&mut value, u64_to_string(i));
            vector::push_back(&mut property_keys, key);
            vector::push_back(&mut property_values, value);
            i = i + 1;
        };

        did::add_service_with_properties_entry(&did_signer, fragment, service_type, service_endpoint, property_keys, property_values);
    }

    // Test fragment length limit: 128 bytes should work, 129 bytes should fail
    #[test]
    fun test_fragment_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Create a 128-byte fragment (should succeed)
        let fragment_128 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 128) {
            std::string::append_utf8(&mut fragment_128, b"a");
            i = i + 1;
        };

        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");
        did::add_service_entry(&did_signer, fragment_128, service_type, service_endpoint);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorFragmentTooLong, location = did)]
    fun test_fragment_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Create a 129-byte fragment (should fail)
        let fragment_129 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 129) {
            std::string::append_utf8(&mut fragment_129, b"a");
            i = i + 1;
        };

        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");
        did::add_service_entry(&did_signer, fragment_129, service_type, service_endpoint);
    }

    // Test string length limits for service type and endpoint
    #[test]
    #[expected_failure(abort_code = did::ErrorStringTooLong, location = did)]
    fun test_service_type_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Create a 513-byte service type (should fail)
        let service_type_513 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 513) {
            std::string::append_utf8(&mut service_type_513, b"a");
            i = i + 1;
        };

        let fragment = std::string::utf8(b"test-service");
        let service_endpoint = std::string::utf8(b"https://example.com");
        did::add_service_entry(&did_signer, fragment, service_type_513, service_endpoint);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorStringTooLong, location = did)]
    fun test_service_endpoint_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // Create a 513-byte service endpoint (should fail)
        let service_endpoint_513 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 513) {
            std::string::append_utf8(&mut service_endpoint_513, b"a");
            i = i + 1;
        };

        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint_513);
    }

    // Test property key/value string length limits
    #[test]
    #[expected_failure(abort_code = did::ErrorStringTooLong, location = did)]
    fun test_property_key_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");

        // Create a 513-byte property key (should fail)
        let property_key_513 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 513) {
            std::string::append_utf8(&mut property_key_513, b"a");
            i = i + 1;
        };

        let property_keys = vector[property_key_513];
        let property_values = vector[std::string::utf8(b"value")];
        did::add_service_with_properties_entry(&did_signer, fragment, service_type, service_endpoint, property_keys, property_values);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorStringTooLong, location = did)]
    fun test_property_value_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");

        // Create a 513-byte property value (should fail)
        let property_value_513 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 513) {
            std::string::append_utf8(&mut property_value_513, b"a");
            i = i + 1;
        };

        let property_keys = vector[std::string::utf8(b"key")];
        let property_values = vector[property_value_513];
        did::add_service_with_properties_entry(&did_signer, fragment, service_type, service_endpoint, property_keys, property_values);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorStringTooLong, location = did)]
    fun test_update_service_property_key_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // First add a service
        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");
        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);

        // Now update with oversized property key (should fail)
        let new_service_type = std::string::utf8(b"UpdatedService");
        let new_service_endpoint = std::string::utf8(b"https://updated.example.com");

        let new_property_key_513 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 513) {
            std::string::append_utf8(&mut new_property_key_513, b"a");
            i = i + 1;
        };

        let new_property_keys = vector[new_property_key_513];
        let new_property_values = vector[std::string::utf8(b"value")];
        did::update_service_entry(&did_signer, fragment, new_service_type, new_service_endpoint, new_property_keys, new_property_values);
    }

    #[test]
    #[expected_failure(abort_code = did::ErrorStringTooLong, location = did)]
    fun test_update_service_property_value_exceeds_length_limit() {
        let (_creator_signer, _addr, _pk, did_object_id) = did_test_common::setup_did_test_with_creation();
        let did_doc = did::get_did_document_by_object_id(did_object_id);
        let did_addr = did::get_did_address(did_doc);
        let did_signer = account::create_signer_for_testing(did_addr);

        // First add a service
        let fragment = std::string::utf8(b"test-service");
        let service_type = std::string::utf8(b"TestService");
        let service_endpoint = std::string::utf8(b"https://example.com");
        did::add_service_entry(&did_signer, fragment, service_type, service_endpoint);

        // Now update with oversized property value (should fail)
        let new_service_type = std::string::utf8(b"UpdatedService");
        let new_service_endpoint = std::string::utf8(b"https://updated.example.com");

        let new_property_value_513 = std::string::utf8(b"");
        let i = 0u64;
        while (i < 513) {
            std::string::append_utf8(&mut new_property_value_513, b"a");
            i = i + 1;
        };

        let new_property_keys = vector[std::string::utf8(b"key")];
        let new_property_values = vector[new_property_value_513];
        did::update_service_entry(&did_signer, fragment, new_service_type, new_service_endpoint, new_property_keys, new_property_values);
    }
}


