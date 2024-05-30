// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::chain_id {

    use moveos_std::object;

    friend rooch_framework::genesis;

    const CHAIN_ID_LOCAL: u64 = 4;
    const CHAIN_ID_DEV: u64 = 3;
    const CHAIN_ID_TEST: u64 = 2;
    const CHAIN_ID_MAIN: u64 = 1;

    /// The ChainID in the global storage
    struct ChainID has key,store {
        id: u64
    }

    public(friend) fun genesis_init(_genesis_account: &signer, chain_id: u64){
        let chain_id = ChainID{
            id: chain_id
        };
        let obj = object::new_named_object(chain_id);
        object::to_frozen(obj);
    }

    public fun id(self: &ChainID) : u64 {
        self.id
    }

    public fun borrow() : &ChainID {
        let object_id = object::named_object_id<ChainID>();
        let obj = object::borrow_object<ChainID>(object_id);
        object::borrow(obj)
    }

    public fun chain_id() : u64 {
        let chain_id = borrow();
        chain_id.id
    }

    public fun is_local() : bool {
        chain_id() == CHAIN_ID_LOCAL
    }

    public fun is_dev() : bool {
        chain_id() == CHAIN_ID_DEV
    }

    public fun is_local_or_dev() : bool {
        let id = chain_id();
        id == CHAIN_ID_LOCAL || id == CHAIN_ID_DEV
    }

    public fun is_test() : bool {
        chain_id() == CHAIN_ID_TEST
    }

    public fun is_main() : bool {
        chain_id() == CHAIN_ID_MAIN
    }
}
