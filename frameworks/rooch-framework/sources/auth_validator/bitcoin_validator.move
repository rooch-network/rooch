// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::bitcoin_validator {

    use std::vector;
    use rooch_framework::multichain_address::{Self, MultiChainAddress};
    use rooch_framework::multichain_address::mapping_to_rooch_address;
    use moveos_std::hex;
    use moveos_std::tx_context;
    use moveos_std::features;
    use moveos_std::hash;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_payload;
    use rooch_framework::auth_validator;
    use rooch_framework::auth_payload::AuthPayload;
    use rooch_framework::bitcoin_address;

    /// there defines auth validator id for each auth validator
    const BITCOIN_AUTH_VALIDATOR_ID: u64 = 1;

    struct BitcoinValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        BITCOIN_AUTH_VALIDATOR_ID
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(payload: AuthPayload, tx_hash: vector<u8>) {

        // tx hash in use wallet signature is hex
        let tx_hex = hex::encode(tx_hash);
        let tx_hex_len = (vector::length(&tx_hex));

        let sign_info_prefix = auth_payload::sign_info_prefix(payload);
        let sign_info_prefix_len = (vector::length(&sign_info_prefix));

        let sign_info = auth_payload::sign_info(payload);
        let sign_info_len = (vector::length(&sign_info));

        assert!(
            sign_info_len + tx_hex_len <= 255,
            auth_validator::error_invalid_authenticator()
        );

        // append tx hash
        let full_tx = vector<u8>[];

        if (sign_info_prefix_len > 0) {
            vector::insert(&mut sign_info_prefix, 0, (sign_info_prefix_len as u8));
            vector::append(&mut full_tx, sign_info_prefix);
        };

        let sign_info_insert_index = 0u64;
        if (sign_info_prefix_len > 0) {
            sign_info_insert_index = sign_info_prefix_len + 1;
        };

        if (vector::length(&sign_info) > 0) {
            vector::insert(&mut full_tx, sign_info_insert_index, ((sign_info_len + tx_hex_len) as u8));
            vector::append(&mut full_tx, sign_info);
            vector::append(&mut full_tx, tx_hex);
        } else {
            vector::insert(&mut full_tx, sign_info_insert_index, (tx_hex_len as u8));
            vector::append(&mut full_tx, tx_hex);
        };
        // append tx hash end
        // The Bitcoin wallet uses sha2_256 twice, the `ecdsa_k1::verify` function also does sha2_256 once
        let full_tx_hash = hash::sha2_256(full_tx);
        assert!(
            ecdsa_k1::verify(
                &auth_payload::sign(payload),
                &auth_payload::public_key(payload),
                &full_tx_hash,
                ecdsa_k1::sha256()
            ),
            auth_validator::error_invalid_authenticator()
        );
    }

    public fun validate(authenticator_payload: vector<u8>): MultiChainAddress {
        features::ensure_testnet_enabled();

        let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = auth_payload::from_bytes(authenticator_payload);

        validate_signature(payload, tx_hash);

        let from_address_in_payload = auth_payload::from_address(payload);
        let bitcoin_addr = bitcoin_address::new(&from_address_in_payload);
        let multi_chain_addr = multichain_address::from_bitcoin(bitcoin_addr);
        // Check if the address and public key are related
        assert!(
            bitcoin_address::verify_with_pk(&from_address_in_payload, &auth_payload::public_key(payload)),
            auth_validator::error_invalid_authenticator()
        );

        let rooch_addr = mapping_to_rooch_address(multi_chain_addr);


        // Check if the sender is related to the Rooch address
        assert!(
            sender == rooch_addr,
            auth_validator::error_invalid_authenticator()
        );

        multi_chain_addr
    }

    fun pre_execute() {}

    fun post_execute() {
    }

    #[test]
    fun test_validate_signature_success() {
        let tx_hash = x"21756929c0b93b54daf211d0b7607a5876bf9d34bb069d3330eef198d23ae1f0";
        let auth_payload_bytes = x"40a761d2cb97cde5535e65f918d111061207e16701f4cfc24fdcf3db474bbfae0f0e751a47c1536ea1bfc6860ca647238c234b86d02fbba6853e34a3c845b643c918426974636f696e205369676e6564204d6573736167653a0a9d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a21038e3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db8833e626331703878706a706b6339757a6a3264657863786a67397377386c786a6538357861343037307a7063797335383965337266366b3230716d36676a7274";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 1002)]
    fun test_validate_signature_fail() {
        let tx_hash = x"22756929c0b93b54daf211d0b7607a5876bf9d34bb069d3330eef198d23ae1f0";
        let auth_payload_bytes = x"40a761d2cb97cde5535e65f918d111061207e16701f4cfc24fdcf3db474bbfae0f0e751a47c1536ea1bfc6860ca647238c234b86d02fbba6853e34a3c845b643c918426974636f696e205369676e6564204d6573736167653a0a9d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a21038e3d29b653e40f5b620f9443ee05222d1e40be58f544b6fed3d464edd54db8833e626331703878706a706b6339757a6a3264657863786a67397377386c786a6538357861343037307a7063797335383965337266366b3230716d36676a7274";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }
}
