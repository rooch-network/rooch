// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.
module rooch_nursery::ethereum_validator {

    use std::vector;
    use std::string;
    use rooch_framework::multichain_address::{Self, MultiChainAddress};
    use moveos_std::tx_context;
    use moveos_std::features;
    use rooch_framework::auth_payload::{AuthPayload};
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_validator;
    use rooch_framework::ethereum_address::{Self, ETHAddress};
    use rooch_framework::auth_payload;

    /// there defines auth validator id for each blockchain
    const ETHEREUM_AUTH_VALIDATOR_ID: u64 = 1;

    struct EthereumValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        ETHEREUM_AUTH_VALIDATOR_ID
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(payload: AuthPayload, tx_hash: vector<u8>): ETHAddress {

        let message = auth_payload::encode_full_message(&payload, tx_hash);

        let pk = ecdsa_k1::ecrecover(&auth_payload::signature(payload), &message, ecdsa_k1::keccak256());
        assert!(
            vector::length(&pk) == ecdsa_k1::public_key_length(),
            auth_validator::error_invalid_authenticator()
        );

        let address = ethereum_address::new(pk);
        assert!(
            ethereum_address::as_bytes(&address) == string::bytes(&auth_payload::from_address(payload)),
            auth_validator::error_invalid_authenticator()
        );

        address
    }

    public fun validate(authenticator_payload: vector<u8>): MultiChainAddress {
        features::ensure_testnet_enabled();
        
        //let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = auth_payload::from_bytes(authenticator_payload);
        let eth_addr = validate_signature(payload, tx_hash);
        let multi_chain_addr = multichain_address::from_eth(eth_addr);
        
        //TODO check if the sender is related to the eth address

        // Check if the sender is related to the Rooch address
        // assert!(
        //     sender == rooch_addr,
        //     auth_validator::error_invalid_authenticator()
        // );

        multi_chain_addr
    }

    fun pre_execute() {}

    fun post_execute() {}

    #[test]
    fun test_validate_signature_success() {
        let tx_hash = x"7190467737b5da545c362dfa804ddeaab8a858d0829260440ea44856ed4ca3ca";
        let auth_payload_bytes = x"419ecf38ec41946dfb50c2e0c2fe2dcc40a69c36b684c6a00bfc6ef26908b018a739f49249c614b77576066aa8a8b844559c02de024874a5b4267e6b00e8615ccf011d19457468657265756d205369676e6564204d6573736167653a0a3232319d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a0014442e5c30b5f6a8ae12d5b72e4db68ed6ecfdff25";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 1002)]
    fun test_validate_signature_fail() {
        let tx_hash = x"7290467737b5da545c362dfa804ddeaab8a858d0829260440ea44856ed4ca3ca";
        let auth_payload_bytes = x"419ecf38ec41946dfb50c2e0c2fe2dcc40a69c36b684c6a00bfc6ef26908b018a739f49249c614b77576066aa8a8b844559c02de024874a5b4267e6b00e8615ccf011d19457468657265756d205369676e6564204d6573736167653a0a3232319d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a0014442e5c30b5f6a8ae12d5b72e4db68ed6ecfdff25";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }
}
