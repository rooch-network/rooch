/// This module defines a struct storing the metadata of the block and new block events.
module rooch_framework::block {
    use std::error;
    use std::signer;
    use rooch_framework::core_addresses;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    // use rooch_framework::timestamp;

    friend rooch_framework::genesis;

    /// Should be in-sync with BlockResource rust struct
    // TODO BlockResource structure
    struct BlockResource has key {
        /// Height of the current block
        height: u64,
        /// Time period between epochs.
        epoch_interval: u64,
    }

    /// Epoch interval cannot be 0.
    const EZeroEpochInterval: u64 = 1;
    const EBlockResourceAlreadyExist: u64 = 2;


    /// This can only be called during Genesis.
    public(friend) fun initialize(ctx: &mut StorageContext, account: &signer, epoch_interval_millisecs: u64) {
        core_addresses::assert_rooch_genesis(account);
        assert!(epoch_interval_millisecs > 0, error::invalid_argument(EZeroEpochInterval));
        assert!(!account_storage::global_exists<BlockResource>(ctx,signer::address_of(account)), error::invalid_state(EBlockResourceAlreadyExist));

        account_storage::global_move_to<BlockResource>(
            ctx,
            account,
            BlockResource {
                height: 0,
                epoch_interval: epoch_interval_millisecs,
            }
        );
    }

    /// Update the epoch interval.
    public fun update_epoch_interval_millisecs(
        ctx: &mut StorageContext,
        account: &signer,
        new_epoch_interval: u64,
    ) {
        core_addresses::assert_rooch_genesis(account);
        assert!(new_epoch_interval > 0, error::invalid_argument(EZeroEpochInterval));

        // let block_resource = borrow_global_mut<BlockResource>(core_addresses::genesis_address());
        let block_resource = account_storage::global_borrow_mut<BlockResource>(ctx, core_addresses::genesis_address());
        let _old_epoch_interval = block_resource.epoch_interval;
        block_resource.epoch_interval = new_epoch_interval;
    }

    #[view]
    /// Return epoch interval in seconds.
    public fun get_epoch_interval_secs(ctx: &mut StorageContext): u64 {
        account_storage::global_borrow<BlockResource>(ctx,core_addresses::genesis_address()).epoch_interval / 1000
    }

    /// Set the metadata for the current block.
    /// The runtime always runs this before executing the transactions in a block.
    // TODO blcok preduce logic
    fun block_prologue(
        vm: signer,
        _hash: address,
        _epoch: u64,
        _round: u64,
        _proposer: address,
        _timestamp: u64
    )  {
        // Operational constraint: can only be invoked by the VM.
        core_addresses::assert_vm(&vm);
    }

    #[view]
    /// Get the current block height
    public fun get_current_block_height(ctx: &mut StorageContext): u64 {
        account_storage::global_borrow<BlockResource>(ctx,core_addresses::genesis_address()).height
    }
}
