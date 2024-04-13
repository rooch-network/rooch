// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::ethereum_validator {

    use std::vector;
    use std::option::{Self, Option};
    use std::signer;
    use rooch_framework::multichain_address;
    use rooch_framework::multichain_address::MultiChainAddress;
    use moveos_std::hex;
    use moveos_std::tx_context;
    use moveos_std::signer::module_signer;
    use moveos_std::features;
    use rooch_framework::auth_payload::{AuthPayload};
    use rooch_framework::account_authentication;
    use rooch_framework::ecdsa_k1;
    use rooch_framework::auth_validator;
    use rooch_framework::ethereum_address::{Self, ETHAddress};
    use rooch_framework::auth_payload;

    /// there defines auth validator id for each blockchain
    const ETHEREUM_AUTH_VALIDATOR_ID: u64 = 1;

    // error code
    const ErrorInvalidPublicKeyLength: u64 = 1;

    struct EthereumValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        ETHEREUM_AUTH_VALIDATOR_ID
    }

    public entry fun rotate_authentication_key_entry(
        account: &signer,
        public_key: vector<u8>
    ) {
        // compare newly passed public key with Ethereum public key length to ensure it's compatible
        assert!(
            vector::length(&public_key) == ecdsa_k1::public_key_length(),
            ErrorInvalidPublicKeyLength
        );

        // User can rotate the authentication key arbitrarily, so we do not need to check the new public key with the account address.
        let authentication_key = public_key_to_authentication_key(public_key);
        let account_addr = signer::address_of(account);
        rotate_authentication_key(account_addr, authentication_key);
    }

    fun rotate_authentication_key(account_addr: address, authentication_key: vector<u8>) {
        account_authentication::rotate_authentication_key<EthereumValidator>(account_addr, authentication_key);
    }

    public entry fun remove_authentication_key_entry(account: &signer) {
        account_authentication::remove_authentication_key<EthereumValidator>(signer::address_of(account));
    }

    public fun public_key_to_address(public_key: vector<u8>): ETHAddress {
        ethereum_address::new(public_key)
    }

    /// Get the authentication key of the given public key.
    public fun public_key_to_authentication_key(public_key: vector<u8>): vector<u8> {
        let addr = public_key_to_address(public_key);
        ethereum_address::into_bytes(addr)
    }

    /// Get the authentication key option of the given account.
    public fun get_authentication_key_option_from_account(addr: address): Option<vector<u8>> {
        account_authentication::get_authentication_key<EthereumValidator>(addr)
    }

    /// The authentication key exists in account or not.
    public fun is_authentication_key_in_account(addr: address): bool {
        option::is_some(&get_authentication_key_option_from_account(addr))
    }

    /// Extract the authentication key of the authentication key option.
    public fun get_authentication_key_from_account(addr: address): vector<u8> {
        option::extract(&mut get_authentication_key_option_from_account(addr))
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(payload: AuthPayload, tx_hash: vector<u8>): ETHAddress {

        // tx hash in use wallet signature is hex
        let tx_hex = hex::encode(tx_hash);
        let tx_hex_len = (vector::length(&tx_hex) as u8);

        let sign_info_prefix = auth_payload::sign_info_prefix(payload);
        let sign_info = auth_payload::sign_info(payload);

        // append tx hash
        let full_tx = vector<u8>[];

        let sign_info_prefix_len = (vector::length(&sign_info_prefix) as u8);
        if (sign_info_prefix_len > 0) {
            vector::append(&mut full_tx, sign_info_prefix);
        };

        if (vector::length(&sign_info) > 0) {
            vector::append(&mut full_tx, sign_info);
            vector::append(&mut full_tx, tx_hex);
        } else {
            vector::insert(&mut full_tx, (sign_info_prefix_len as u64) + 1, tx_hex_len);
            vector::append(&mut full_tx, tx_hex);
        };
        // append tx hash end

        let pk = ecdsa_k1::ecrecover(&auth_payload::sign(payload), &full_tx, ecdsa_k1::keccak256());
        assert!(
            vector::length(&pk) == ecdsa_k1::public_key_length(),
            auth_validator::error_invalid_authenticator()
        );

        let address = ethereum_address::new(pk);
        assert!(
            *ethereum_address::as_bytes(&address) == auth_payload::from_address(payload),
            auth_validator::error_invalid_authenticator()
        );

        address
    }

    public fun validate(authenticator_payload: vector<u8>): MultiChainAddress {
        features::ensure_testnet_enabled();
        
        let sender = tx_context::sender();
        let tx_hash = tx_context::tx_hash();
        let payload = auth_payload::from_bytes(authenticator_payload);
        let ethAddress = validate_signature(payload, tx_hash);

        if (!is_authentication_key_in_account(sender)) {
            // For the first invocation, the default auth key is the eth address
            // Save to context and record after the transaction is executed
            tx_context::add_attribute_via_system(&module_signer<EthereumValidator>(), ethAddress);
        } else {
            let authKey = get_authentication_key_from_account(sender);
            assert!(
                *ethereum_address::as_bytes(&ethAddress) == authKey,
                auth_validator::error_invalid_authenticator()
            );
        };

        multichain_address::from_eth(ethAddress)
    }

    fun pre_execute() {}

    fun post_execute() {
        let account_addr = tx_context::sender();
        if (!is_authentication_key_in_account(account_addr)) {
            let authKey = option::extract(&mut tx_context::get_attribute<ETHAddress>());
            rotate_authentication_key(tx_context::sender(), *ethereum_address::as_bytes(&authKey));
            let auth_key_in_account = get_authentication_key_from_account(account_addr);
            std::debug::print(&auth_key_in_account);
        }
    }

    #[test]
    fun test_validate_signature_success() {
        let tx_hash = x"0f8fda918b3ac80ebc5384fa2ec8d8486b013e4a104a90216438955dece2e22a";
        let auth_payload_bytes = x"41b9d45b14976ce71590961fdb0ecff95936bec3ce6f79c77a191cb098b876e5b3737720c6aecc87b3a7d0b4e47db7380de5c9c0016fd6e2ccc213144bfbd171b7011d19457468657265756d205369676e6564204d6573736167653a0a3232319d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a0000140738eae71452937fe41d9de7594bbb8264572b4a";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = 1002)]
    fun test_validate_signature_fail() {
        let tx_hash = x"deb8d910fcb86e00545234c7a10da3d6cf51e08014299d473cf07545899b1d25";
        let auth_payload_bytes = x"41b9d45b14976ce71590961fdb0ecff95936bec3ce6f79c77a191cb098b876e5b3737720c6aecc87b3a7d0b4e47db7380de5c9c0016fd6e2ccc213144bfbd171b7011d19457468657265756d205369676e6564204d6573736167653a0a3232319d0157656c636f6d6520746f206c6f63616c686f73740a596f752077696c6c20617574686f72697a652073657373696f6e3a0a53636f70653a0a3078343965653363663137613031376233333161623262386134643430656363393730366633323835363266396462363363626136323561396331303663646633353a3a2a3a3a2a0a54696d654f75743a313230300a526f6f636820747820686173683a0a0000140738eae71452937fe41d9de7594bbb8264572b4a";
        let payload = auth_payload::from_bytes(auth_payload_bytes);

        validate_signature(payload, tx_hash);
    }
}
