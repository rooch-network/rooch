// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction_validator {
    use std::option;
    use moveos_std::timestamp;
    use moveos_std::signer::module_signer;
    use moveos_std::tx_context;
    use moveos_std::tx_result;
    use moveos_std::account;
    use moveos_std::gas_schedule;
    use moveos_std::tx_meta;
    use rooch_framework::account as account_entry;
    use rooch_framework::account_authentication;
    use rooch_framework::auth_validator::{Self, TxValidateResult};
    use rooch_framework::auth_validator_registry;
    use rooch_framework::session_key;
    use rooch_framework::chain_id;
    use rooch_framework::transaction_fee;
    use rooch_framework::gas_coin;
    use rooch_framework::transaction::{Self, TransactionSequenceInfo};
    use rooch_framework::session_validator;
    use rooch_framework::bitcoin_validator;
    use rooch_framework::address_mapping;
    use rooch_framework::account_coin_store;
    use rooch_framework::builtin_validators;
    use rooch_framework::onchain_config;
    use rooch_framework::coin;
    use rooch_framework::bitcoin_address;
    use rooch_framework::webauthn_validator;

    const MAX_U64: u128 = 18446744073709551615;


    /// Just using to get module signer
    struct TransactionValidatorPlaceholder {}

    /// This function is for Rooch to validate the transaction sender's authenticator.
    /// If the authenticator is invaid, abort this function.
    public(friend) fun validate(
        chain_id: u64,
        auth_validator_id: u64,
        authenticator_payload: vector<u8>
    ): TxValidateResult {

        // === validate the chain id ===
        assert!(
            chain_id == chain_id::chain_id(),
            auth_validator::error_validate_bad_chain_id(),
        );

        // === validate the sequence number ===
        let tx_sequence_number = tx_context::sequence_number();
        assert!(
            (tx_sequence_number as u128) < MAX_U64,
            auth_validator::error_validate_sequence_number_too_big(),
        );
        let sender = tx_context::sender();
        let account_sequence_number = account::sequence_number(sender);
        assert!(
            tx_sequence_number >= account_sequence_number,
            auth_validator::error_validate_sequence_number_too_old(),
        );

        // Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new.
        assert!(
            tx_sequence_number == account_sequence_number,
            auth_validator::error_validate_sequence_number_too_new(),
        );

        // === validate gas ===
        let max_gas_amount = tx_context::max_gas_amount();
        let gas = transaction_fee::calculate_gas(max_gas_amount);

        let max_gas_amount_config = gas_schedule::max_gas_amount();
        assert!(
            max_gas_amount <= max_gas_amount_config,
            auth_validator::error_validate_max_gas_amount_exceeded(),
        );

        let gas_balance = gas_coin::balance(sender);

        // we do not need to check the gas balance in local or dev chain
        if(!chain_id::is_local_or_dev()){
            assert!(
                gas_balance >= gas,
                auth_validator::error_validate_cant_pay_gas_deposit(),
            );
        };

        // === validate the authenticator ===

        // Try the built-in auth validator first
        let (bitcoin_address_opt, session_key, auth_validator)= if (auth_validator_id == session_validator::auth_validator_id()){
            let session_key = session_validator::validate(authenticator_payload);
            let bitcoin_address = address_mapping::resolve_bitcoin(sender);
            (bitcoin_address, option::some(session_key), option::none())
        }else if (auth_validator_id == bitcoin_validator::auth_validator_id()){
            let bitcoin_address = bitcoin_validator::validate(authenticator_payload);
            (option::some(bitcoin_address), option::none(), option::none())
        }else if (auth_validator_id == webauthn_validator::auth_validator_id()){
            let session_key = webauthn_validator::validate(authenticator_payload);
            let bitcoin_address = address_mapping::resolve_bitcoin(sender);
            (bitcoin_address, option::some(session_key), option::none())
        }else{
            let auth_validator = auth_validator_registry::borrow_validator(auth_validator_id);
            let validator_id = auth_validator::validator_id(auth_validator);
            // The third-party auth validator must be installed to the sender's account
            assert!(builtin_validators::is_builtin_auth_validator(validator_id) || account_authentication::is_auth_validator_installed(sender, validator_id),
                    auth_validator::error_validate_not_installed_auth_validator());
            let bitcoin_address = address_mapping::resolve_bitcoin(sender);
            (bitcoin_address, option::none(), option::some(*auth_validator))
        };
        // We enable the smart contract account(DID account) to send transaction via session key, and the smart contract account does not have a bitcoin address.
        // But for compatibility, we still need to return a empty bitcoin address in the TxValidateResult.
        let bitcoin_address = if (option::is_some(&bitcoin_address_opt)) {
            option::destroy_some(bitcoin_address_opt)
        }else {
            bitcoin_address::empty()
        };
        auth_validator::new_tx_validate_result(auth_validator_id, auth_validator, session_key, bitcoin_address)
    }

    /// Transaction pre_execute function.
    /// Execute before the transaction is executed, automatically called by the MoveOS VM.
    /// This function is for Rooch to auto create account and address maping.
    /// The system call transaction do not execute the pre_execute and post_execute function.
    fun pre_execute(
    ) {
        let sender = tx_context::sender();
        //Auto create account if not exist
        if (!account::exists_at(sender)) {
            account_entry::create_account(sender);
        };
        //if the chain is local or dev, give the sender some RGAS
        if (chain_id::is_local_or_dev() && gas_coin::balance(sender) == 0) {
            //10000 RGAS
            let init_gas = 1000_000_000_000u256;
            gas_coin::faucet(sender, init_gas); 
        };
        let bitcoin_addr_opt = auth_validator::get_bitcoin_address_from_ctx_option();
        if (option::is_some(&bitcoin_addr_opt)) {
            let bitcoin_addr = option::destroy_some(bitcoin_addr_opt);
            address_mapping::bind_bitcoin_address_internal(sender, bitcoin_addr); 
        };
        let tx_sequence_info = tx_context::get_attribute<TransactionSequenceInfo>();
        if (option::is_some(&tx_sequence_info)) {
            let tx_sequence_info = option::extract(&mut tx_sequence_info);
            let tx_timestamp = transaction::tx_timestamp(&tx_sequence_info);
            let module_signer = module_signer<TransactionValidatorPlaceholder>();
            timestamp::try_update_global_time(&module_signer, tx_timestamp);
        };
        let gas_payment_account = tx_context::tx_gas_payment_account();
        let max_gas_amount = tx_context::max_gas_amount();
        let gas = transaction_fee::calculate_gas(max_gas_amount);
        let gas_coin = gas_coin::deduct_gas(gas_payment_account, gas);
        transaction_fee::deposit_fee(gas_coin);
    }

    /// Transaction post_execute function.
    /// Execute after the transaction is executed, automatically called by the MoveOS VM.
    /// This function is for Rooch to update the sender's sequence number and pay the gas fee.
    fun post_execute(
    ) {
        let sender = tx_context::sender();

        // Active the session key

        let session_key_opt = auth_validator::get_session_key_from_ctx_option();
        if (option::is_some(&session_key_opt)) {
            let session_key = option::extract(&mut session_key_opt);
            session_key::active_session_key(session_key);
        };
        // Increment sequence number
        let system = module_signer<TransactionValidatorPlaceholder>();
        account::increment_sequence_number_for_system(&system, sender);

        let tx_result = tx_context::tx_result();
        let gas_payment_account = tx_context::tx_gas_payment_account();
        let gas_used = tx_result::gas_used(&tx_result);
        let gas_used_after_scale = transaction_fee::calculate_gas(gas_used);

        let max_gas_amount = tx_context::max_gas_amount();
        let paid_gas = transaction_fee::calculate_gas(max_gas_amount);

        let tx_meta = tx_context::tx_meta();
        let function_call_opt = tx_meta::function_meta(&tx_meta);
        let contract_address = if (option::is_some(&function_call_opt)) {
            let function_call = option::borrow(&function_call_opt);
            *tx_meta::function_meta_module_address(function_call)
        } else {
            //If it is not a function call, we use the framework address as the contract address
            @rooch_framework
        };
        let sequencer_address = onchain_config::sequencer();
        let remaining_gas_coin = transaction_fee::distribute_fee(paid_gas, gas_used_after_scale, contract_address, sequencer_address);
        if (coin::value(&remaining_gas_coin) > 0) {
            account_coin_store::deposit(gas_payment_account, remaining_gas_coin);
        }else{
            coin::destroy_zero(remaining_gas_coin);
        }
    }
}
