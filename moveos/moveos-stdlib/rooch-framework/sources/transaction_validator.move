module rooch_framework::transaction_validator {
    use std::error;
    use std::signer;
    use rooch_framework::account;
    use rooch_framework::chain_id;
    use rooch_framework::core_addresses;
    use rooch_framework::timestamp;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;

    friend rooch_framework::genesis;

    /// This holds information that will be picked up by the VM to call the
    /// correct chain-specific prologue and epilogue functions
    struct TransactionValidation has key {
        module_addr: address,
        module_name: vector<u8>,
        script_prologue_name: vector<u8>,
        module_prologue_name: vector<u8>,
        multi_agent_prologue_name: vector<u8>,
        user_epilogue_name: vector<u8>,
    }

    const MAX_U64: u128 = 18446744073709551615;

    /// Transaction exceeded its allocated max gas
    const EOUT_OF_GAS: u64 = 6;

    /// Prologue errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system.
    const EPrologueInvalidAccountAuthKey: u64 = 1001;
    const EPrologueSequenceNuberTooOld: u64 = 1002;
    const EPrologueSequenceNumberTooNew: u64 = 1003;
    const EPrologueAccountDoesNotExist: u64 = 1004;
    const EPrologueCantPayGasDeposit: u64 = 1005;
    const EPrologueTransactionExpired: u64 = 1006;
    const EPrologueBadChainId: u64 = 1007;
    const EPrologueSequenceNumberTooBig: u64 = 1008;
    const EPrologueSecondaryKeysAddressesCountMismatch: u64 = 1009;

    /// Only called during genesis to initialize system resources for this module.
    public(friend) fun initialize(
        ctx: &mut StorageContext,
        account: &signer,
        script_prologue_name: vector<u8>,
        module_prologue_name: vector<u8>,
        multi_agent_prologue_name: vector<u8>,
        user_epilogue_name: vector<u8>,
    ) {
        core_addresses::assert_rooch_genesis(account);
        account_storage::global_move_to<TransactionValidation>(
            ctx,
            account,
            TransactionValidation {
                module_addr: core_addresses::genesis_address(),
                module_name: b"transaction_validator",
                script_prologue_name,
                module_prologue_name,
                multi_agent_prologue_name,
                user_epilogue_name,
            }
        );
    }

    fun prologue_common(
        ctx: &mut StorageContext,
        sender: signer,
        tx_sequence_number: u64,
        tx_authentication_key: vector<u8>,
        _tx_gas_price: u64,
        _tx_max_gas_units: u64,
        tx_expiration_time: u64,
        chain_id: u8,
    ) {
        assert!(
            timestamp::now_seconds(ctx) < tx_expiration_time,
            error::invalid_argument(EPrologueTransactionExpired),
        );
        assert!(chain_id::get(ctx) == chain_id, error::invalid_argument(EPrologueBadChainId));

        let transaction_sender = signer::address_of(&sender);
        assert!(account::exists_at(ctx, transaction_sender), error::invalid_argument(EPrologueAccountDoesNotExist));
        assert!(
            tx_authentication_key == account::get_authentication_key(ctx, transaction_sender),
            error::invalid_argument(EPrologueInvalidAccountAuthKey),
        );

        assert!(
            (tx_sequence_number as u128) < MAX_U64,
            error::out_of_range(EPrologueSequenceNumberTooBig)
        );

        let account_sequence_number = account::sequence_number(ctx, transaction_sender);
        assert!(
            tx_sequence_number >= account_sequence_number,
            error::invalid_argument(EPrologueSequenceNuberTooOld)
        );

        // [PCA12]: Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new by [PCA11].
        assert!(
            tx_sequence_number == account_sequence_number,
            error::invalid_argument(EPrologueSequenceNumberTooNew)
        );

        // TODO check transaction gas fee
        // let max_transaction_fee = tx_gas_price * tx_max_gas_units;
        // assert!(
        //     coin::is_account_registered<...>(transaction_sender),
        //     error::invalid_argument(EPrologueCantPayGasDeposit),
        // );
        // let balance = coin::balance<...>(transaction_sender);
        // assert!(balance >= max_transaction_fee, error::invalid_argument(EPrologueCantPayGasDeposit));
    }

    fun module_prologue(
        ctx: &mut StorageContext,
        sender: signer,
        tx_sequence_number: u64,
        tx_public_key: vector<u8>,
        tx_gas_price: u64,
        tx_max_gas_units: u64,
        tx_expiration_time: u64,
        chain_id: u8,
    ) {
        prologue_common(ctx, sender, tx_sequence_number, tx_public_key, tx_gas_price, tx_max_gas_units, tx_expiration_time, chain_id)
    }


    /// Epilogue function is run after a transaction is successfully executed.
    /// Called by the Adapter
    fun epilogue(
        ctx: &mut StorageContext,
        account: signer,
        _tx_sequence_number: u64,
        tx_gas_price: u64,
        tx_max_gas_units: u64,
        gas_units_remaining: u64
    ) {
        assert!(tx_max_gas_units >= gas_units_remaining, error::invalid_argument(EOUT_OF_GAS));
        let gas_used = tx_max_gas_units - gas_units_remaining;

        assert!(
            (tx_gas_price as u128) * (gas_used as u128) <= MAX_U64,
            error::out_of_range(EOUT_OF_GAS)
        );
        let addr = signer::address_of(&account);
        //TODO handle transaction gas fee
        // let transaction_fee_amount = tx_gas_price * gas_used;
        // // it's important to maintain the error code consistent with vm
        // // to do failed transaction cleanup.
        // assert!(
        //     coin::balance<...>(addr) >= transaction_fee_amount,
        //     error::out_of_range(EPrologueCantPayGasDeposit),
        // );
        //
        // if (features::collect_and_distribute_gas_fees()) {
        //     // If transaction fees are redistributed to validators, collect them here for
        //     // later redistribution.
        //     transaction_fee::collect_fee(addr, transaction_fee_amount);
        // } else {
        //     // Otherwise, just burn the fee.
        //     // TODO: this branch should be removed completely when transaction fee collection
        //     // is tested and is fully proven to work well.
        //     transaction_fee::burn_fee(addr, transaction_fee_amount);
        // };

        // Increment sequence number
        account::increment_sequence_number(ctx, addr);
    }
}
