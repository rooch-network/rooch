// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::bitcoin_validator {

    use std::vector;
    use std::option::{Self, Option};
    use std::signer;
    use moveos_std::context::{Self, Context};
    use rooch_framework::account_authentication;
    use rooch_framework::ecdsa_k1_recoverable;
    use rooch_framework::auth_validator;
    use rooch_framework::ethereum_address::{Self, ETHAddress};

    /// there defines auth validator id for each blockchain
    const BITCOIN_AUTH_VALIDATOR_ID: u64 = 2;

    // error code
    const ErrorInvalidPublicKeyLength: u64 = 1;

    struct BitcoinValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        BITCOIN_AUTH_VALIDATOR_ID
    }

    public entry fun rotate_authentication_key_entry(
        ctx: &mut Context,
        account: &signer,
        public_key: vector<u8>
    ) {
        // compare newly passed public key with Ethereum public key length to ensure it's compatible
        assert!(
            vector::length(&public_key) == ecdsa_k1_recoverable::public_key_length(),
            ErrorInvalidPublicKeyLength
        );

        // User can rotate the authentication key arbitrarily, so we do not need to check the new public key with the account address.
        let authentication_key = public_key_to_authentication_key(public_key);
        let account_addr = signer::address_of(account);
        rotate_authentication_key(ctx, account_addr, authentication_key);
    }

    fun rotate_authentication_key(ctx: &mut Context, account_addr: address, authentication_key: vector<u8>) {
        account_authentication::rotate_authentication_key<BitcoinValidator>(ctx, account_addr, authentication_key);
    }

    public entry fun remove_authentication_key_entry(ctx: &mut Context, account: &signer) {
        account_authentication::remove_authentication_key<BitcoinValidator>(ctx, signer::address_of(account));
    }

    public fun get_public_key_from_authenticator_payload(authenticator_payload: &vector<u8>): vector<u8> {
        let public_key = vector::empty<u8>();
        let i = ecdsa_k1_recoverable::signature_length() ;
        let public_key_position = ecdsa_k1_recoverable::signature_length() + ecdsa_k1_recoverable::public_key_length();
        while (i < public_key_position) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut public_key, *value);
            i = i + 1;
        };
        std::debug::print(&public_key);
        public_key
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

    /// Get the authentication key of the given authenticator from authenticator_payload.
    public fun get_authentication_key_from_authenticator_payload(authenticator_payload: &vector<u8>): vector<u8> {
        let public_key = get_public_key_from_authenticator_payload(authenticator_payload);
        let addr = public_key_to_address(public_key);
        ethereum_address::into_bytes(addr)
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
    public fun get_authentication_key_option_from_account(ctx: &Context, addr: address): Option<vector<u8>> {
        account_authentication::get_authentication_key<BitcoinValidator>(ctx, addr)
    }

    /// The authentication key exists in account or not.
    public fun is_authentication_key_in_account(ctx: &Context, addr: address): bool {
        option::is_some(&get_authentication_key_option_from_account(ctx, addr))
    }

    /// Extract the authentication key of the authentication key option.
    public fun get_authentication_key_from_account(ctx: &Context, addr: address): vector<u8> {
        option::extract(&mut get_authentication_key_option_from_account(ctx, addr))
    }

    /// Only validate the authenticator's signature.
    public fun validate_signature(authenticator_payload: &vector<u8>, tx_hash: &vector<u8>) {
        std::debug::print(tx_hash);
        assert!(
            ecdsa_k1_recoverable::verify(
                &get_signature_from_authenticator_payload(authenticator_payload),
                // &get_public_key_from_authenticator_payload(authenticator_payload),
                tx_hash,
                1
            ),
            auth_validator::error_invalid_authenticator()
        );
    }

    public fun validate(ctx: &Context, authenticator_payload: vector<u8>) {
        let tx_hash = context::tx_hash(ctx);
        validate_signature(&authenticator_payload, &tx_hash);

        // TODO compare the auth_key from lsthe payload with the auth_key from the account
    }

    fun pre_execute(
        _ctx: &mut Context,
    ) {}

    fun post_execute(
        ctx: &mut Context,
    ) {
        let account_addr = context::sender(ctx);
        if (is_authentication_key_in_account(ctx, account_addr)) {
            let auth_key_in_account = get_authentication_key_from_account(ctx, account_addr);
            std::debug::print(&auth_key_in_account);
        }
    }

    // todo: how to convert publick to address and test
    // this test ensures that the Ethereum public_key_to_address function is compatible with the one in the rust code
    #[test]
    fun test_public_key_to_address() {
        let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
        let addr = public_key_to_address(public_key);
        let address_bytes = ethereum_address::into_bytes(addr);
        let expected_address = x"1a642f0e3c3af545e7acbd38b07251b3990914f1";
        assert!(address_bytes == expected_address, 1000);
    }
}
