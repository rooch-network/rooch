// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module contains the error code for auth_validator module
/// The auth_validator implementation should contain the following functions
/// public fun validate(authenticator_payload: vector<u8>)
/// fun pre_execute()
/// fun post_execute()
module rooch_framework::auth_validator {
    use std::option::{Self, Option};
    use moveos_std::tx_context;
    use rooch_framework::bitcoin_address::BitcoinAddress;

    friend rooch_framework::auth_validator_registry;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::session_key;

    /// The function must be executed after the transaction is validated
    const ErrorMustExecuteAfterValidate: u64 = 1;

    /// The AuthKey in transaction's authenticator do not match with the sender's account auth key
    const ErrorValidateInvalidAccountAuthKey: u64 = 1001;
    /// InvalidAuthenticator, include invalid signature
    const ErrorValidateInvalidAuthenticator: u64 = 1002;

    public fun error_invalid_account_auth_key(): u64 {
        ErrorValidateInvalidAccountAuthKey
    }

    public fun error_invalid_authenticator(): u64 {
        ErrorValidateInvalidAuthenticator
    }

    /// The Authentication Validator
    struct AuthValidator has store, copy, drop {
        id: u64,
        module_address: address,
        module_name: std::string::String,
    }

    public(friend) fun new_auth_validator(
        id: u64,
        module_address: address,
        module_name: std::string::String
    ): AuthValidator {
        AuthValidator {
            id: id,
            module_address: module_address,
            module_name: module_name,
        }
    }

    public fun validator_id(validator: &AuthValidator): u64 {
        validator.id
    }

    public fun validator_module_address(validator: &AuthValidator): address {
        validator.module_address
    }

    public fun validator_module_name(validator: &AuthValidator): std::string::String {
        validator.module_name
    }

    /// The Transaction Validate Result
    /// this result will be stored in the TxContext
    struct TxValidateResult has copy, store, drop {
        /// The auth validator's id that validate the transaction
        auth_validator_id: u64,
        auth_validator: Option<AuthValidator>,
        session_key: Option<vector<u8>>,
        bitcoin_address: BitcoinAddress,
    }

    public(friend) fun new_tx_validate_result(
        auth_validator_id: u64,
        auth_validator: Option<AuthValidator>,
        session_key: Option<vector<u8>>,
        bitcoin_address: BitcoinAddress,
    ): TxValidateResult {
        TxValidateResult {
            auth_validator_id: auth_validator_id,
            auth_validator: auth_validator,
            session_key: session_key,
            bitcoin_address: bitcoin_address,
        }
    }

    /// Get the TxValidateResult from the TxContext, Only can be called after the transaction is validated
    public(friend) fun get_validate_result_from_ctx(): TxValidateResult {
        let validate_result_opt = tx_context::get_attribute<TxValidateResult>();
        assert!(option::is_some(&validate_result_opt), ErrorMustExecuteAfterValidate);
        option::extract(&mut validate_result_opt)
    }

    /// Get the auth validator's id from the TxValidateResult in the TxContext
    public(friend) fun get_validator_id_from_ctx(): u64 {
        let validate_result = get_validate_result_from_ctx();
        validate_result.auth_validator_id
    }

    /// Get the session key from the TxValidateResult in the TxContext
    /// If the TxValidateResult is None or SessionKey is None, return None
    public(friend) fun get_session_key_from_ctx_option(): Option<vector<u8>> {
        let validate_result_opt = tx_context::get_attribute<TxValidateResult>();
        if (option::is_some(&validate_result_opt)) {
            let validate_result = option::extract(&mut validate_result_opt);
            validate_result.session_key 
        }else {
            option::none<vector<u8>>()
        }
    }

    /// The current tx is validate via the session key or not
    public(friend) fun is_validate_via_session_key(): bool {
        option::is_some(&get_session_key_from_ctx_option())
    }

    /// Get the session key from the TxValidateResult in the TxContext
    /// Only can be called after the transaction is validated
    public(friend) fun get_session_key_from_ctx(): vector<u8> {
        assert!(is_validate_via_session_key(), ErrorMustExecuteAfterValidate);
        option::extract(&mut get_session_key_from_ctx_option())
    }

    public(friend) fun get_bitcoin_address_from_ctx(): BitcoinAddress {
        let validate_result = get_validate_result_from_ctx();
        validate_result.bitcoin_address
    }
}
