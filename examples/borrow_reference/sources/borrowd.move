// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::borrowd {
    use moveos_std::account_storage;
    use moveos_std::context::{Self, Context};

    struct BorrowCapability has key, copy, store {}

    struct DataStore has key, copy, store {
        v: u8
    }

    public fun new_borrow_cap() : BorrowCapability {
        BorrowCapability {}
    }

    public fun new_data_store() : DataStore {
        DataStore {
            v: 0
        }
    }

    public fun do_immutable_borrow(
        ctx: &Context,
        _borrow_cap: &BorrowCapability,
    ) {
        let addr = context::sender(ctx);
        account_storage::global_exists<BorrowCapability>(ctx, addr);
    }

    public fun do_mutable_borrow(
        ctx: &mut Context,
        addr: address,
        data_store: &mut DataStore,
    ) {
        if (account_storage::global_exists<DataStore>(ctx, addr)) {
            data_store.v = data_store.v + 1
        }
    }
}
