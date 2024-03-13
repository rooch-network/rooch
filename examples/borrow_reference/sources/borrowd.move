// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::borrowd {
    use moveos_std::account;
    

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
        
        _borrow_cap: &BorrowCapability,
    ) {
        let addr = moveos_std::tx_context::sender();
        account::exists_resource<BorrowCapability>(addr);
    }

    public fun do_mutable_borrow(
        
        addr: address,
        data_store: &mut DataStore,
    ) {
        if (account::exists_resource<DataStore>(addr)) {
            data_store.v = data_store.v + 1
        }
    }
}
