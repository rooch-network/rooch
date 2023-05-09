/// The chain id distinguishes between different chains (e.g., testnet and the main network).
/// One important role is to prevent transactions intended for one chain from being executed on another.
/// This code provides a container for storing a chain id and functions to initialize and get it.
module rooch_framework::chain_id {
    use std::signer;
    use std::error;
    use rooch_framework::core_addresses;
    friend rooch_framework::genesis;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    #[test_only]
    use moveos_std::storage_context;
    #[test_only]
    use std::debug;

    struct ChainId has key {
        id: u8
    }

    const MAIN_CHAIN_ID: u8 = 1;
    const DEV_CHAIN_ID: u8 = 181;
    const TEST_CHAIN_ID: u8 = 182;

    const EChainIdAlreadyExist: u64 = 1;

    /// Only called during genesis.
    /// Publish the chain ID under the genesis account
    public(friend) fun initialize(ctx: &mut StorageContext, account: &signer, id: u8) {
        core_addresses::assert_rooch_genesis(account);
        assert!(!account_storage::global_exists<ChainId>(ctx,signer::address_of(account)), error::invalid_state(EChainIdAlreadyExist));
        account_storage::global_move_to<ChainId>(
            ctx,
            account,
            ChainId {
                id,
            }
        );
    }

    #[view]
    /// Return the chain ID of this instance.
    public fun get(ctx: &mut StorageContext): u8 {
        account_storage::global_borrow<ChainId>(ctx, @rooch_framework).id
    }

    public fun is_dev(ctx: &mut StorageContext): bool {
        get(ctx) == DEV_CHAIN_ID
    }

    public fun is_test(ctx: &mut StorageContext): bool {
        get(ctx) == TEST_CHAIN_ID
    }

    public fun is_main(ctx: &mut StorageContext): bool {
        get(ctx) == MAIN_CHAIN_ID
    }

    #[test_only]
    public fun initialize_for_test(ctx: &mut StorageContext, rooch_framework: &signer, id: u8) {
        initialize(ctx, rooch_framework, id);
    }

    // #[test]
    #[test(genesis = @rooch_framework)]
    fun test_get(genesis: &signer) {
        let genesis_address = signer::address_of(genesis);
        let ctx = storage_context::test_context(genesis_address);
        debug::print(&10000);
        debug::print(&ctx);
        initialize_for_test(&mut ctx, genesis, DEV_CHAIN_ID);
        assert!(get(&mut ctx) == DEV_CHAIN_ID, 1001);
        storage_context::drop_storage_context(ctx);
    }
}
