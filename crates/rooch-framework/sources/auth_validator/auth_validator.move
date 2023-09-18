/// This module contains the error code for auth_validator module
/// The auth_validator implementation should contain the following functions
/// public fun validate(ctx: &StorageContext, authenticator_payload: vector<u8>)
/// fun pre_execute(ctx: &mut StorageContext)
/// fun post_execute(ctx: &mut StorageContext)
module rooch_framework::auth_validator {
    use std::error;
    use std::option::{Self, Option};
    use moveos_std::storage_context::{Self, StorageContext};

    friend rooch_framework::auth_validator_registry;
    friend rooch_framework::transaction_validator;

    /// The function must be executed after the transaction is validated
    const ErrorMustExecuteAfterValidate: u64 = 1;

    /// The AuthKey in transaction's authenticator do not match with the sender's account auth key
    const ErrorValidateInvalidAccountAuthKey: u64 = 1001;
    /// InvalidAuthenticator, include invalid signature
    const ErrorValidateInvalidAuthenticator: u64 = 1002;

    public fun error_invalid_account_auth_key(): u64 {
        error::invalid_argument(ErrorValidateInvalidAccountAuthKey)
    }

    public fun error_invalid_authenticator(): u64 {
        error::invalid_argument(ErrorValidateInvalidAuthenticator)
    }

    /// The Authentication Validator
    struct AuthValidator has store, copy, drop {
        id: u64,
        module_address: address,
        module_name: std::ascii::String,
    }

    public(friend) fun new_auth_validator(
        id: u64,
        module_address: address,
        module_name: std::ascii::String
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

    public fun validator_module_name(validator: &AuthValidator): std::ascii::String {
        validator.module_name
    }

    /// The Transaction Validate Result
    /// this result will be stored in the TxContext
    struct TxValidateResult has copy, store, drop {
        /// The auth validator's auth validator id that validate the transaction
        auth_validator_id: u64,
        auth_validator: Option<AuthValidator>,
        session_key: Option<vector<u8>>,
    }

    public(friend) fun new_tx_validate_result(
        auth_validator_id: u64,
        auth_validator: Option<AuthValidator>,
        session_key: Option<vector<u8>>
    ): TxValidateResult {
        TxValidateResult {
            auth_validator_id: auth_validator_id,
            auth_validator: auth_validator,
            session_key: session_key,
        }
    }

    /// Get the TxValidateResult from the TxContext, Only can be called after the transaction is validated
    public fun get_validate_result_from_tx_ctx(ctx: &StorageContext): TxValidateResult {
        let validate_result_opt = storage_context::get<TxValidateResult>(ctx);
        assert!(option::is_some(&validate_result_opt), error::invalid_state(ErrorMustExecuteAfterValidate));
        option::extract(&mut validate_result_opt)
    }

    /// Get the auth validator's auth validator id from the TxValidateResult in the TxContext
    public fun get_validator_id_from_tx_ctx(ctx: &StorageContext): u64 {
        let validate_result = get_validate_result_from_tx_ctx(ctx);
        validate_result.auth_validator_id
    }

    /// Get the session key from the TxValidateResult in the TxContext
    /// If the TxValidateResult is None or SessionKey is None, return None
    public fun get_session_key_from_tx_ctx_option(ctx: &StorageContext): Option<vector<u8>> {
        let validate_result_opt = storage_context::get<TxValidateResult>(ctx);
        if (option::is_some(&validate_result_opt)) {
            let validate_result = option::extract(&mut validate_result_opt);
            validate_result.session_key 
        }else {
            option::none<vector<u8>>()
        }
    }

    /// The current tx is validate via the session key or not
    public fun is_validate_via_session_key(ctx: &StorageContext): bool {
        option::is_some(&get_session_key_from_tx_ctx_option(ctx))
    }

    /// Get the session key from the TxValidateResult in the TxContext
    /// Only can be called after the transaction is validated
    public fun get_session_key_from_tx_ctx(ctx: &StorageContext): vector<u8> {
        assert!(is_validate_via_session_key(ctx), error::invalid_state(ErrorMustExecuteAfterValidate));
        option::extract(&mut get_session_key_from_tx_ctx_option(ctx))
    }
}