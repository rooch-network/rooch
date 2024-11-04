// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ton blockchain auth validator.
module rooch_framework::ton_validator {

    use std::string;
    use std::option;
    
    use moveos_std::signer;
    use moveos_std::hex;
    use moveos_std::tx_context;
    
    use rooch_framework::auth_validator;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::ton_proof::{Self, TonProofData};
    use rooch_framework::ton_address::{TonAddress};
    use rooch_framework::address_mapping;

    friend rooch_framework::genesis;

    /// there defines auth validator id for each blockchain
    const TON_AUTH_VALIDATOR_ID: u64 = 4;

    const ErrorGenesisInitError: u64 = 1;
    const ErrorAddressMappingRecordNotFound: u64 = 2;

    struct TonValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        TON_AUTH_VALIDATOR_ID
    }

    public(friend) fun genesis_init(){
        let system = signer::module_signer<TonValidator>();
        let id = auth_validator_registry::register_by_system_with_id<TonValidator>(&system, TON_AUTH_VALIDATOR_ID);
        assert!(id == TON_AUTH_VALIDATOR_ID, ErrorGenesisInitError);
    }

    fun validate_signature(ton_address: &TonAddress, proof_data: &TonProofData, tx_hash: vector<u8>) {
        assert!(ton_proof::verify_proof(ton_address, proof_data), auth_validator::error_validate_invalid_authenticator());
        let proof = ton_proof::proof(proof_data);
        let tx_hash_from_payload = ton_proof::payload_tx_hash(proof);

        //make sure the tx_hash is included in the payload, maybe we need to add more info in the payload?
        let tx_hex = hex::encode(tx_hash);
        assert!(&tx_hex == string::bytes(&tx_hash_from_payload), auth_validator::error_validate_invalid_authenticator());
    }

    public fun validate(authenticator_payload: vector<u8>) {
        let proof_data = ton_proof::decode_proof_data(authenticator_payload);
        let sender = tx_context::sender();
        let sender_ton_addr_opt = address_mapping::resolve_to_ton_address(sender);
        assert!(option::is_some(&sender_ton_addr_opt), ErrorAddressMappingRecordNotFound);

        let sender_ton_addr = option::destroy_some(sender_ton_addr_opt);
        let tx_hash = tx_context::tx_hash();
        validate_signature(&sender_ton_addr, &proof_data, tx_hash);
    }
}
