// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ton blockchain auth validator.
module rooch_nursery::ton_validator {

    use std::string;
    use std::option;
    
    use moveos_std::hex;
    use moveos_std::tx_context;
    
    use rooch_framework::auth_validator;
    
    use rooch_nursery::ton_proof::{Self, TonProofData};
    use rooch_nursery::ton_address::{TonAddress};
    use rooch_nursery::ton_address_mapping;

    /// there defines auth validator id for each blockchain
    const TON_AUTH_VALIDATOR_ID: u64 = 3;

    const ErrorAddressMappingRecordNotFound: u64 = 1;

    struct TonValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        TON_AUTH_VALIDATOR_ID
    }


    public fun validate_signature(ton_address: &TonAddress, proof_data: &TonProofData, tx_hash: vector<u8>) {
        assert!(ton_proof::verify_proof(ton_address, proof_data), auth_validator::error_validate_invalid_authenticator());
        let proof = ton_proof::proof(proof_data);
        let payload = ton_proof::payload(proof);

        //make sure the tx_hash is included in the payload, maybe we need to add more info in the payload?
        let tx_hex = hex::encode(tx_hash);
        assert!(&tx_hex == string::bytes(payload), auth_validator::error_validate_invalid_authenticator());
    }

    public fun validate(authenticator_payload: vector<u8>) {
        let proof_data = ton_proof::decode_proof_data(authenticator_payload);
        let sender = tx_context::sender();
        let sender_ton_addr_opt = ton_address_mapping::resolve_to_ton_address(sender);
        assert!(option::is_some(&sender_ton_addr_opt), ErrorAddressMappingRecordNotFound);

        let sender_ton_addr = option::destroy_some(sender_ton_addr_opt);
        let tx_hash = tx_context::tx_hash();
        validate_signature(&sender_ton_addr, &proof_data, tx_hash);
    }
}
