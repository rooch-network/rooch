// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::did_limits_test {
    use std::vector;
    use moveos_std::account;
    use rooch_framework::did;
    use rooch_framework::did_test_common;

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
}


