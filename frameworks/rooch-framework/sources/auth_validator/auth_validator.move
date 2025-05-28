// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module contains the error code for auth_validator module
/// The auth_validator implementation should contain the following functions
/// public fun validate(authenticator_payload: vector<u8>)

module rooch_framework::auth_validator {
    use std::string;
    use std::option::{Self, Option};
    use moveos_std::tx_context;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};

    friend rooch_framework::auth_validator_registry;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::session_key;

    /// The function must be executed after the transaction is validated
    const ErrorMustExecuteAfterValidate: u64 = 1;

    /// Validate errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system.
    const ErrorValidateSequenceNuberTooOld: u64 = 1001;
    const ErrorValidateSequenceNumberTooNew: u64 = 1002;
    const ErrorValidateAccountDoesNotExist: u64 = 1003;
    const ErrorValidateCantPayGasDeposit: u64 = 1004;
    const ErrorValidateTransactionExpired: u64 = 1005;
    const ErrorValidateBadChainId: u64 = 1006;
    const ErrorValidateSequenceNumberTooBig: u64 = 1007;
    const ErrorValidateMaxGasAmountExceeded: u64 = 1008;
    /// The AuthKey in transaction's authenticator do not match with the sender's account auth key
    const ErrorValidateInvalidAccountAuthKey: u64 = 1009;
    /// InvalidAuthenticator, include invalid signature
    const ErrorValidateInvalidAuthenticator: u64 = 1010;
    /// The authenticator's auth validator id is not installed to the sender's account
    const ErrorValidateNotInstalledAuthValidator: u64 = 1011;
    /// The session is expired
    const ErrorValidateSessionIsExpired: u64 = 1012;
    /// The function call is beyond the session's scope
    const ErrorValidateFunctionCallBeyondSessionScope: u64 = 1013;

    public fun error_validate_sequence_number_too_old(): u64 {
        ErrorValidateSequenceNuberTooOld
    }

    public fun error_validate_sequence_number_too_new(): u64 {
        ErrorValidateSequenceNumberTooNew
    }

    public fun error_validate_account_does_not_exist(): u64 {
        ErrorValidateAccountDoesNotExist
    }

    public fun error_validate_cant_pay_gas_deposit(): u64 {
        ErrorValidateCantPayGasDeposit
    }

    public fun error_validate_transaction_expired(): u64 {
        ErrorValidateTransactionExpired
    }

    public fun error_validate_bad_chain_id(): u64 {
        ErrorValidateBadChainId
    }

    public fun error_validate_sequence_number_too_big(): u64 {
        ErrorValidateSequenceNumberTooBig
    }

    public fun error_validate_max_gas_amount_exceeded(): u64 {
        ErrorValidateMaxGasAmountExceeded
    }
    
    public fun error_validate_invalid_account_auth_key(): u64 {
        ErrorValidateInvalidAccountAuthKey
    }

    public fun error_validate_invalid_authenticator(): u64 {
        ErrorValidateInvalidAuthenticator
    }

    public fun error_validate_not_installed_auth_validator(): u64 {
        ErrorValidateNotInstalledAuthValidator
    }

    public fun error_validate_session_is_expired(): u64 {
        ErrorValidateSessionIsExpired
    }

    public fun error_validate_function_call_beyond_session_scope(): u64 {
        ErrorValidateFunctionCallBeyondSessionScope
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
        module_name: string::String
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
    public fun get_session_key_from_ctx_option(): Option<vector<u8>> {
        let validate_result_opt = tx_context::get_attribute<TxValidateResult>();
        if (option::is_some(&validate_result_opt)) {
            let validate_result = option::extract(&mut validate_result_opt);
            validate_result.session_key 
        }else {
            option::none<vector<u8>>()
        }
    }

    /// The current tx is validate via the session key or not
    public fun is_validate_via_session_key(): bool {
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

    public fun get_bitcoin_address_from_ctx_option(): Option<BitcoinAddress> {
        let validate_result_opt = tx_context::get_attribute<TxValidateResult>();
        if (option::is_some(&validate_result_opt)) {
            let validate_result = option::extract(&mut validate_result_opt);
            if (bitcoin_address::is_empty(&validate_result.bitcoin_address)) {
                option::none<BitcoinAddress>()
            }else {
                option::some(validate_result.bitcoin_address)
            }
        }else {
            option::none<BitcoinAddress>()
        }
    }

    #[test_only]
    /// Create a test TxValidateResult and set it in the context
    public fun set_tx_validate_result_for_testing(
        auth_validator_id: u64,
        auth_validator: Option<AuthValidator>,
        session_key: Option<vector<u8>>,
        bitcoin_address: BitcoinAddress,
    ) {
        let result = new_tx_validate_result(auth_validator_id, auth_validator, session_key, bitcoin_address);
        if (tx_context::contains_attribute<TxValidateResult>()) {
            std::debug::print(&string::utf8(b"Reset TxValidateResult in tx_context\n"));
            tx_context::remove_attribute_for_testing<TxValidateResult>();
        };
        tx_context::set_attribute_for_testing(result);
    }

    #[test_only]
    /// Create a simple TxValidateResult for basic testing (with default Bitcoin address)
    public fun set_simple_tx_validate_result_for_testing(session_key: Option<vector<u8>>) {
        let bitcoin_address = rooch_framework::bitcoin_address::random_address_for_testing();
        let auth_validator = option::none<AuthValidator>();
        set_tx_validate_result_for_testing(0, auth_validator, session_key, bitcoin_address);
    }

    #[test_only]
    /// Create a TxValidateResult with a random Bitcoin address for testing
    public fun set_random_tx_validate_result_for_testing(session_key: Option<vector<u8>>) {
        let bitcoin_address = rooch_framework::bitcoin_address::random_address_for_testing();
        let auth_validator = option::none<AuthValidator>();
        set_tx_validate_result_for_testing(0, auth_validator, session_key, bitcoin_address);
    }

    #[test_only]
    /// Create a test AuthValidator
    public fun create_test_auth_validator(id: u64): AuthValidator {
        new_auth_validator(id, @0x1, std::string::utf8(b"test_validator"))
    }
}
