// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Bitcoin multisign auth validator
module bitcoin_move::bitcoin_multisign_validator{

    use std::vector;
    use moveos_std::tx_context;
    use moveos_std::hash;
    use moveos_std::signer;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::auth_validator;
    use rooch_framework::auth_payload;
    use bitcoin_move::multisign_account;

    friend bitcoin_move::genesis;

    const ErrorGenesisInitError: u64 = 1;

    /// there defines auth validator id for each auth validator
    const BITCOIN_MULTISIGN_VALIDATOR_ID: u64 = 2;

    struct BitcoinMultisignValidator has store, drop {}


    public fun auth_validator_id(): u64 {
        BITCOIN_MULTISIGN_VALIDATOR_ID
    }

    public(friend) fun genesis_init(){
        let system = signer::module_signer<BitcoinMultisignValidator>();
        let id = auth_validator_registry::register_by_system<BitcoinMultisignValidator>(&system);
        assert!(id == BITCOIN_MULTISIGN_VALIDATOR_ID, ErrorGenesisInitError);
    }

    /// Init function called by upgrade. This module is upgrade after genesis, so we provide this function for upgrade.
    /// When rest the genesis, we can remove this function.
    public fun init_for_upgrade(){
        if(!auth_validator_registry::is_registered<BitcoinMultisignValidator>()){
            genesis_init();
        }
    }

    /// Only validate the authenticator's signature.
    fun validate_signatures(signatures: &vector<vector<u8>>, public_keys: &vector<vector<u8>>, message: vector<u8>) {
        
        let signature_len = vector::length(signatures);
        assert!(
            signature_len == vector::length(public_keys),
            auth_validator::error_validate_invalid_authenticator()
        );

        // The Bitcoin wallet uses sha2_256 twice, the `ecdsa_k1::verify` function also does sha2_256 once
        let message_hash = hash::sha2_256(message);
        let i = 0;
        while (i < signature_len) {
            assert!(
                ecdsa_k1::verify(
                    vector::borrow(signatures,i),
                    vector::borrow(public_keys,i),
                    &message_hash,
                    ecdsa_k1::sha256()
                ),
                auth_validator::error_validate_invalid_authenticator()
            );
            i = i + 1;
        }
    }

    fun validate_multisign_account(multisign_address: address, public_keys: &vector<vector<u8>>) {
        assert!(
            multisign_account::is_multisign_account(multisign_address),
            auth_validator::error_validate_invalid_authenticator()
        );
        let pubkey_len = vector::length(public_keys);
        assert!(pubkey_len >= multisign_account::threshold(multisign_address), auth_validator::error_validate_invalid_authenticator());
        let i = 0;
        while (i < pubkey_len) {
            assert!(
                multisign_account::is_participant_via_public_key(multisign_address, vector::borrow(public_keys,i)),
                auth_validator::error_validate_invalid_authenticator()
            );
            i = i + 1;
        }    
    }

    public fun validate(authenticator_payload: vector<u8>) {
        let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = auth_payload::multisign_from_bytes(authenticator_payload);
        let message = auth_payload::multisign_encode_full_message(&payload, tx_hash);
        let signatures = auth_payload::multisign_signatures(&payload);
        let public_keys = auth_payload::multisign_public_keys(&payload);
        validate_multisign_account(sender, public_keys);
        validate_signatures(signatures, public_keys, message);
    }

}