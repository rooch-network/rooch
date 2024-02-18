// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::chain_id {

    use moveos_std::object_id;
    use moveos_std::context::{Self, Context};
    use moveos_std::object;

    friend rooch_framework::genesis;

    const CHAIN_ID_LOCAL: u64 = 20230104;
    const CHAIN_ID_DEV: u64 = 20230103;
    const CHAIN_ID_TEST: u64 = 20230102;
    const CHAIN_ID_MAIN: u64 = 20230101;

    /// The ChainID in the global storage
    struct ChainID has key,store {
        id: u64
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer, chain_id: u64){
        let chain_id = ChainID{
            id: chain_id
        };
        let obj = context::new_named_object(ctx, chain_id);
        object::to_frozen(obj);
    }

    public fun id(self: &ChainID) : u64 {
        self.id
    }

    public fun borrow(_ctx: &Context) : &ChainID {
        let object_id = object_id::named_object_id<ChainID>();
        let obj = object::borrow_object<ChainID>(object_id);
        object::borrow(obj)
    }

    public fun chain_id(ctx: &Context) : u64 {
        let chain_id = borrow(ctx);
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
