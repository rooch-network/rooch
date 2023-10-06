module rooch_framework::chain_id {
    
    use moveos_std::context::Context;
    use moveos_std::account_storage;

    friend rooch_framework::genesis;

    const CHAIN_ID_LOCAL: u64 = 20230104;
    const CHAIN_ID_DEV: u64 = 20230103;
    const CHAIN_ID_TEST: u64 = 20230102;
    const CHAIN_ID_MAIN: u64 = 20230101;

    /// The ChainID in the global storage
    struct ChainID has key,store,copy,drop {
        id: u64
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer, chain_id: u64){
        let chain_id = ChainID{
            id: chain_id
        };
        account_storage::global_move_to(ctx, genesis_account, chain_id);
    }

    public fun chain_id(ctx: &Context) : u64 {
        let chain_id = account_storage::global_borrow<ChainID>(ctx, @rooch_framework);
        chain_id.id
    }

    public fun is_local(ctx: &Context) : bool {
        chain_id(ctx) == CHAIN_ID_LOCAL
    }

    public fun is_dev(ctx: &Context) : bool {
        chain_id(ctx) == CHAIN_ID_DEV
    }

    public fun is_test(ctx: &Context) : bool {
        chain_id(ctx) == CHAIN_ID_TEST
    }

    public fun is_main(ctx: &Context) : bool {
        chain_id(ctx) == CHAIN_ID_MAIN
    }
}