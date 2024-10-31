// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.
module rooch_nursery::ethereum_validator {

    use std::vector;
    use std::string;
    
    use moveos_std::tx_context;
    use moveos_std::features;
    use moveos_std::signer;

    use rooch_framework::auth_payload::{AuthPayload};
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_validator;
    use rooch_framework::ethereum_address::{Self, ETHAddress};
    use rooch_framework::auth_payload;
    use rooch_framework::multichain_address;
    use rooch_framework::auth_validator_registry;

    friend rooch_nursery::genesis;

    /// there defines auth validator id for each blockchain
    const ETHEREUM_AUTH_VALIDATOR_ID: u64 = 3;

    const ErrorGenesisInitError: u64 = 1;

    struct EthereumValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        ETHEREUM_AUTH_VALIDATOR_ID
    }

    public(friend) fun genesis_init(){
        let system = signer::module_signer<EthereumValidator>();
        let id = auth_validator_registry::register_by_system<EthereumValidator>(&system);
        assert!(id == ETHEREUM_AUTH_VALIDATOR_ID, ErrorGenesisInitError);
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(payload: &AuthPayload, tx_hash: vector<u8>): ETHAddress {

        let message = auth_payload::encode_full_message(payload, tx_hash);

        let pk = ecdsa_k1::ecrecover(&auth_payload::signature(payload), &message, ecdsa_k1::keccak256());
        assert!(
            vector::length(&pk) == ecdsa_k1::public_key_length(),
            auth_validator::error_validate_invalid_authenticator()
        );

        let address = ethereum_address::new(pk);
        assert!(
            ethereum_address::as_bytes(&address) == string::bytes(&auth_payload::from_address(payload)),
            auth_validator::error_validate_invalid_authenticator()
        );

        address
    }

    public fun validate(authenticator_payload: vector<u8>) {
        features::ensure_testnet_enabled();
        
        //let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = auth_payload::from_bytes(authenticator_payload);
        let eth_addr = validate_signature(&payload, tx_hash);
        let _multi_chain_addr = multichain_address::from_eth(eth_addr);
        
        //TODO check if the sender is related to the eth address

        // Check if the sender is related to the Rooch address
        // assert!(
        //     sender == rooch_addr,
        //     auth_validator::error_validate_invalid_authenticator()
        // );

    }
}
