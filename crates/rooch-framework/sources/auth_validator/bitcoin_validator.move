// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::bitcoin_validator {

    use std::vector;
    use moveos_std::context::{Self, Context};
    use rooch_framework::ecdsa_k1_recoverable;
    use rooch_framework::auth_validator;

    /// there defines auth validator id for each blockchain
    const BITCOIN_AUTH_VALIDATOR_ID: u64 = 2;

    // error code
    const ErrorInvalidPublicKeyLength: u64 = 1;

    struct BitcoinValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        BITCOIN_AUTH_VALIDATOR_ID
    }

    public fun get_signature_from_authenticator_payload(authenticator_payload: &vector<u8>): vector<u8> {
        let sign = vector::empty<u8>();
        let i = 0;
        let signature_position = ecdsa_k1_recoverable::signature_length();
        while (i < signature_position) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut sign, *value);
            i = i + 1;
        };
        std::debug::print(&sign);
        sign
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>) {
        std::debug::print(tx_hash);
        assert!(
            ecdsa_k1_recoverable::verify(
                &get_signature_from_authenticator_payload(authenticator_payload),
                tx_hash,
                ecdsa_k1_recoverable::sha256()
            ),
            auth_validator::error_invalid_authenticator()
        );
    }

    public fun validate(ctx: &Context, authenticator_payload: vector<u8>) {
        let tx_hash = context::tx_hash(ctx);
        validate_signature(&authenticator_payload, &tx_hash);
    }

    fun pre_execute(
        _ctx: &mut Context,
    ) {}

    fun post_execute(
        _ctx: &mut Context,
    ) {
        // TODO: need to bind the address mapping here.
    }
}
