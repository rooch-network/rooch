// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::did_cadop_controller_test {
    use std::string;
    use std::vector;
    use std::option;
    // no extra imports
    use rooch_framework::did;
    use rooch_framework::did_test_common;

    // Happy path: create DID via CADOP with did:bitcoin controller
    #[test]
    fun test_create_did_via_cadop_with_bitcoin_controller_success() {
        // Prepare custodian with CADOP service
        let custodian_signer = did_test_common::setup_custodian_with_cadop_service();

        // Prepare user controller: secp256k1 pubkey and matching bitcoin address
        let (user_pk_multibase, _user_btc_addr) = did_test_common::generate_secp256k1_public_key_and_bitcoin_address();

        // The matching Taproot address string for the hard-coded pubkey in generate_secp256k1_public_key_and_bitcoin_address
        let btc_addr_str = string::utf8(b"bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g");

        // Build controller DID string: did:bitcoin:<addr>
        let controller_did_str = string::utf8(b"did:bitcoin:");
        string::append(&mut controller_did_str, btc_addr_str);

        // Custodian service VM info (secp256k1)
        let custodian_service_pk = did_test_common::generate_test_secp256k1_multibase();
        let custodian_service_vm_type = did::verification_method_type_secp256k1();

        // Use empty custom scopes (default behavior will use provided empty vector)
        let scopes = vector::empty<string::String>();

        // Create DID via new controller-based CADOP entry
        did::create_did_object_via_cadop_with_controller_and_scopes_entry(
            &custodian_signer,
            controller_did_str,
            user_pk_multibase,
            did::verification_method_type_secp256k1(),
            custodian_service_pk,
            custodian_service_vm_type,
            scopes
        );

        // Verify registry now maps controller DID -> one DID
        let dids = did::get_dids_by_controller_string(string::utf8(b"did:bitcoin:bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"));
        assert!(vector::length(&dids) == 1, 10101);

        // Fetch DID document by DID string
        let created_did_str = *vector::borrow(&dids, 0);
        let did_doc = did::get_did_document(created_did_str);

        // Authentication must include account-key
        assert!(did::has_verification_relationship_in_doc(did_doc, &string::utf8(b"account-key"), did::verification_relationship_authentication()), 10102);

        // account-key VM must exist and have expected type (secp256k1)
        let vm_opt = did::doc_verification_method(did_doc, &string::utf8(b"account-key"));
        assert!(option::is_some(&vm_opt), 10103);
        let vm = option::destroy_some(vm_opt);
        assert!(*did::verification_method_type(&vm) == did::verification_method_type_secp256k1(), 10104);
    }

    // Failure: did:bitcoin controller with mismatched address and public key should abort
    #[test]
    #[expected_failure(abort_code = did::ErrorControllerBitcoinAddressMismatch, location = did)]
    fun test_create_did_via_cadop_with_bitcoin_controller_mismatch_address() {
        // Prepare custodian with CADOP service
        let custodian_signer = did_test_common::setup_custodian_with_cadop_service();

        // Prepare user pk (valid secp256k1), but provide a different valid bitcoin address string
        let (user_pk_multibase, _user_btc_addr) = did_test_common::generate_secp256k1_public_key_and_bitcoin_address();

        let other_btc_addr_str = string::utf8(b"1BoatSLRHtKNngkdXEeobR76b53LETtpyT");
        let controller_did_str = string::utf8(b"did:bitcoin:");
        string::append(&mut controller_did_str, other_btc_addr_str);

        let custodian_service_pk = did_test_common::generate_test_secp256k1_multibase();
        let custodian_service_vm_type = did::verification_method_type_secp256k1();
        let scopes = vector::empty<string::String>();

        did::create_did_object_via_cadop_with_controller_and_scopes_entry(
            &custodian_signer,
            controller_did_str,
            user_pk_multibase,
            did::verification_method_type_secp256k1(),
            custodian_service_pk,
            custodian_service_vm_type,
            scopes
        );
    }
}


