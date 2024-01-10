// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::bitcoin_validator {

    use std::vector;
    use rooch_framework::auth_payload::AuthPayload;
    use moveos_std::hex;
    use moveos_std::context::{Self, Context};
    use rooch_framework::hash;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_payload;
    use rooch_framework::auth_validator;
    use rooch_framework::multichain_address::MultiChainAddress;

    /// there defines auth validator id for each blockchain
    const BITCOIN_AUTH_VALIDATOR_ID: u64 = 2;

    // error code
    const ErrorInvalidPublicKeyLength: u64 = 1;

    struct BitcoinValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        BITCOIN_AUTH_VALIDATOR_ID
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(payload: AuthPayload, tx_hash: vector<u8>) {

        // tx hash in use wallet signature is hex
        let tx_hex = hex::encode(tx_hash);
        let tx_hex_len = (vector::length(&tx_hex) as u8);

        let sign_info_prefix = auth_payload::sign_info_prefix(payload);
        let sign_info = auth_payload::sign_info(payload);

        // append tx hash
        let full_tx = vector<u8>[];

        let sign_info_prefix_len = (vector::length(&sign_info_prefix) as u8);
        if (sign_info_prefix_len > 0) {
            vector::insert(&mut sign_info_prefix, 0, sign_info_prefix_len);
            vector::append(&mut full_tx, sign_info_prefix);
        };

        let sign_info_len = (vector::length(&sign_info) as u8);
        if (vector::length(&sign_info) > 0) {
            vector::insert(&mut full_tx, (sign_info_prefix_len as u64) + 1,sign_info_len + tx_hex_len);
            vector::append(&mut full_tx, sign_info);
            vector::append(&mut full_tx, tx_hex);
        } else {
            vector::insert(&mut full_tx, (sign_info_prefix_len as u64) + 1, tx_hex_len);
            vector::append(&mut full_tx, tx_hex);
        };
        // append tx hash end

        // The Bitcoin wallet uses has256 twice
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

    public fun validate(ctx: &Context, authenticator_payload: vector<u8>): MultiChainAddress {
        let tx_hash = context::tx_hash(ctx);
        let payload = auth_payload::from_bytes(authenticator_payload);

        validate_signature(payload, tx_hash);

        auth_payload::multi_address(payload)
    }

    fun pre_execute(
        _ctx: &mut Context,
    ) {}

    fun post_execute(
        _ctx: &mut Context,
    ) {}

    #[test]
    fun test_validate_signature_success() {
        let tx_hash = x"d60d66db3188c8b07f43143e428ff7dd6b9f4bff706586e9d90a8f290374377c";
        let auth_payload_bytes = x"402e8ef34763557c87041d3691ee0bc9cc941885c9c777f93ef6d26b2df8efdb4564cad195f8f339de5c3d2bcbda49b11649163be55051256093f510a32a3d825218426974636f696e205369676e6564204d6573736167653a0a9d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a2103ee14a358f6e924f82f0a263807d585a7a222f14bf45bfbeebaa33991c700d08e1f0000000000000000160200a696edc27ef17e9f079ee07d4d915c23a738a80d00";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 1002)]
    fun test_validate_signature_fail() {
        let tx_hash = x"deb8d910fcb86e00545234c7a10da3d6cf51e08014299d473cf07545899b1d25";
        let auth_payload_bytes = x"402e8ef34763557c87041d3691ee0bc9cc941885c9c777f93ef6d26b2df8efdb4564cad195f8f339de5c3d2bcbda49b11649163be55051256093f510a32a3d825218426974636f696e205369676e6564204d6573736167653a0a9d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a2103ee14a358f6e924f82f0a263807d585a7a222f14bf45bfbeebaa33991c700d08e1f0000000000000000160200a696edc27ef17e9f079ee07d4d915c23a738a80d00";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }
}
