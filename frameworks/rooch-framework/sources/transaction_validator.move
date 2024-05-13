// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction_validator {
    use std::option;
    use moveos_std::signer::module_signer;
    use moveos_std::tx_context;
    use moveos_std::tx_result;
    use moveos_std::account;
    use rooch_framework::account as account_entry;
    use rooch_framework::multichain_address::MultiChainAddress;
    use rooch_framework::address_mapping;
    use rooch_framework::account_authentication;
    use rooch_framework::auth_validator::{Self, TxValidateResult};
    use rooch_framework::auth_validator_registry;
    use rooch_framework::session_key;
    use rooch_framework::chain_id;
    use rooch_framework::transaction_fee;
    use rooch_framework::gas_coin;

    const MAX_U64: u128 = 18446744073709551615;


    /// Transaction exceeded its allocated max gas
    const ErrorOutOfGas: u64 = 1;

    //TODO Migrate the error code to the auth_validator module 
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

    /// The authenticator's auth validator id is not installed to the sender's account
    const ErrorValidateNotInstalledAuthValidator: u64 = 1010;

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
            ErrorValidateBadChainId
        );

        // === validate the sequence number ===
        let tx_sequence_number = tx_context::sequence_number();
        assert!(
            (tx_sequence_number as u128) < MAX_U64,
            ErrorValidateSequenceNumberTooBig
        );
        let sender = tx_context::sender();
        let account_sequence_number = account::sequence_number(sender);
        assert!(
            tx_sequence_number >= account_sequence_number,
            ErrorValidateSequenceNuberTooOld
        );

        // Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new.
        //FIXME we temporarily disable this check, in order to improve the devnet experience
        //Because the devnet will be reset frequently, so the sequence number will be reset to 0
        //But the sequence number(nonce) in MetaMask will not be reset, so the transaction will be rejected 
        // assert!(
        //     tx_sequence_number == account_sequence_number,
        //     ErrorValidateSequenceNumberTooNew
        // );

        // === validate gas ===
        let max_gas_amount = tx_context::max_gas_amount();
        let gas = transaction_fee::calculate_gas(max_gas_amount);

        // We skip the gas check for the new account, for avoid break the current testcase
        // TODO remove the skip afater we provide the gas faucet and update all testcase
        if(account::exists_at(sender)){
            let gas_balance = gas_coin::balance(sender);
            assert!(
                gas_balance >= gas,
                ErrorValidateCantPayGasDeposit
            );
        };

        // === validate the authenticator ===

        // if the authenticator authenticator_payload is session key, validate the session key
        // otherwise return the authentication validator via the auth validator id
        let session_key_option = session_key::validate(auth_validator_id, authenticator_payload);
        if (option::is_some(&session_key_option)) {
            std::debug::print(&auth_validator_id);
            std::debug::print(&session_key_option);
            auth_validator::new_tx_validate_result(auth_validator_id, option::none(), session_key_option)
        } else {
            let auth_validator = auth_validator_registry::borrow_validator(auth_validator_id);
            let validator_id = auth_validator::validator_id(auth_validator);
            // builtin auth validator id do not need to install
            if (!rooch_framework::builtin_validators::is_builtin_auth_validator(auth_validator_id)) {
                assert!(
                    account_authentication::is_auth_validator_installed(sender, validator_id),
                    ErrorValidateNotInstalledAuthValidator
                );
            };
            auth_validator::new_tx_validate_result(auth_validator_id, option::some(*auth_validator), option::none())
        }
    }

    /// Transaction pre_execute function.
    /// Execute before the transaction is executed, automatically called by the MoveOS VM.
    /// This function is for Rooch to auto create account and address maping.
    fun pre_execute(
        
    ) {
        let sender = tx_context::sender();
        //Auto create account if not exist
        if (!account::exists_at(sender)) {
            account_entry::create_account_internal(sender);
            // Auto get gas coin from faucet if not enough
            // TODO remove this after we provide the gas faucet
            //100 RGC
            let init_gas = 100_000_000_000_000_000_000u256;
            gas_coin::faucet(sender, init_gas); 
        }; 
        //the transaction validator will put the multi chain address into the context
        let multichain_address = tx_context::get_attribute<MultiChainAddress>();
        if (option::is_some(&multichain_address)) {
            let multichain_address = option::extract(&mut multichain_address);
            //Auto create address mapping if not exist
            if (!address_mapping::exists_mapping(multichain_address)) {
                address_mapping::bind_no_check(sender, multichain_address);
            };
        };
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
        let gas = transaction_fee::calculate_gas(gas_used);
        let gas_coin = gas_coin::deduct_gas(gas_payment_account, gas);
        transaction_fee::deposit_fee(gas_coin);
    }
}
