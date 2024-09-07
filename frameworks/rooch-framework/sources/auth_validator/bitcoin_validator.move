// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Bitcoin validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::bitcoin_validator {

    use moveos_std::tx_context;
    use moveos_std::hash;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_payload;
    use rooch_framework::auth_validator;
    use rooch_framework::auth_payload::AuthPayload;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};

    friend rooch_framework::transaction_validator;

    /// there defines auth validator id for each auth validator
    const BITCOIN_AUTH_VALIDATOR_ID: u64 = 1;

    struct BitcoinValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        BITCOIN_AUTH_VALIDATOR_ID
    }

    /// Only validate the authenticator's signature.
    fun validate_signature(payload: &AuthPayload, tx_hash: vector<u8>) {

        let message = auth_payload::encode_full_message(payload, tx_hash);

        // The Bitcoin wallet uses sha2_256 twice, the `ecdsa_k1::verify` function also does sha2_256 once
        let message_hash = hash::sha2_256(message);
        assert!(
            ecdsa_k1::verify(
                &auth_payload::signature(payload),
                &auth_payload::public_key(payload),
                &message_hash,
                ecdsa_k1::sha256()
            ),
            auth_validator::error_validate_invalid_authenticator()
        );
    }

    public(friend) fun validate(authenticator_payload: vector<u8>) :BitcoinAddress{

        let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = auth_payload::from_bytes(authenticator_payload);

        validate_signature(&payload, tx_hash);

        let from_address_in_payload = auth_payload::from_address(&payload);
        let bitcoin_addr = bitcoin_address::from_string(&from_address_in_payload);

        // Check if the address and public key are related
        assert!(
            bitcoin_address::verify_with_public_key(&from_address_in_payload, &auth_payload::public_key(&payload)),
            auth_validator::error_validate_invalid_authenticator()
        );

        let rooch_addr = bitcoin_address::to_rooch_address(&bitcoin_addr);

        // Check if the sender is related to the Rooch address
        assert!(
            sender == rooch_addr,
            auth_validator::error_validate_invalid_authenticator()
        );
        bitcoin_addr
    }

    #[test]
    fun test_validate_signature_success() {
        let tx_hash = x"5415b18de0b880bb2af5dfe1ee27fd19ae8a0c99b5328e8b4b44f4c86cc7176a";
        let auth_payload_bytes = x"407e5b0c1da7d2bed7c2497b7c7c46b1a485883029a3bb1479493688ad347bcafa2bd82c6fd9bb2515f9e0c697f621ac0a28fb9f8c0e565d5b6d4e20bf18ce86621a18426974636f696e205369676e6564204d6573736167653a0ae2a201526f6f6368205472616e73616374696f6e3a0a57656c636f6d6520746f20726f6f63685f746573740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078663962313065366337363066316361646365393563363634623361336561643363393835626265396436336264353161396266313736303738356432366131623a3a2a3a3a2a0a54696d654f75743a313030300a21031a446b6ac064acb14687764871dad6c08186a788248d585b3cce69231b48d1382a62633171333234356e706d3430346874667a76756c783676347736356d61717a7536617474716c336677";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(&payload, tx_hash);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 1010)]
    fun test_validate_signature_fail() {
        let tx_hash = x"5515b18de0b880bb2af5dfe1ee27fd19ae8a0c99b5328e8b4b44f4c86cc7176a";
        let auth_payload_bytes = x"407e5b0c1da7d2bed7c2497b7c7c46b1a485883029a3bb1479493688ad347bcafa2bd82c6fd9bb2515f9e0c697f621ac0a28fb9f8c0e565d5b6d4e20bf18ce86621a18426974636f696e205369676e6564204d6573736167653a0ae2a201526f6f6368205472616e73616374696f6e3a0a57656c636f6d6520746f20726f6f63685f746573740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078663962313065366337363066316361646365393563363634623361336561643363393835626265396436336264353161396266313736303738356432366131623a3a2a3a3a2a0a54696d654f75743a313030300a21031a446b6ac064acb14687764871dad6c08186a788248d585b3cce69231b48d1382a62633171333234356e706d3430346874667a76756c783676347736356d61717a7536617474716c336677";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(&payload, tx_hash);
    }
}
