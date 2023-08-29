module rooch_framework::chain_id {
    
    use moveos_std::storage_context::StorageContext;
    use moveos_std::account_storage;

    friend rooch_framework::genesis;

    struct ChainID has key,store,copy,drop {
        id: u64
    }

    public(friend) fun genesis_init(ctx: &mut StorageContext, genesis_account: &signer, chain_id: u64){
        let chain_id = ChainID{
            id: chain_id
        };
        account_storage::global_move_to(ctx, genesis_account, chain_id);
    }

    public fun chain_id(ctx: &StorageContext) : u64 {
        let chain_id = account_storage::global_borrow<ChainID>(ctx, @rooch_framework);
        chain_id.id
    }
}