// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::borrow_test {
    use moveos_std::account;
    use rooch_examples::borrowd::{BorrowCapability, DataStore};
    use rooch_examples::borrowd;
    

    #[test_only]
    use std::signer;

    struct Capabilities has key {
        borrow_cap: BorrowCapability,
    }

    struct DataStoreWrapper has key {
        data_store: DataStore,
    }

    public fun init_borrow(
        
        account: &signer,
    ) {
        let borrow_cap = borrowd::new_borrow_cap();
        let data_store = borrowd::new_data_store();
        account::move_resource_to<Capabilities>(account, Capabilities {
            borrow_cap,
        });
        account::move_resource_to<DataStoreWrapper>(account, DataStoreWrapper {
            data_store,
        });
    }

    public fun borrow(
        
        addr: address,
    ) {
        let cap = account::borrow_mut_resource<Capabilities>(addr);
        borrowd::do_immutable_borrow(&cap.borrow_cap);

        //  Invalid usage of reference as function argument. Cannot transfer a mutable reference that is being borrowed
        // let data_store_warpper = account::borrow_mut_resource<DataStoreWrapper>(addr);
        // borrowd::do_mutable_borrow(addr, &mut data_store_warpper.data_store);
    }

    #[test(alice = @0x11)]
    fun test_borrow(alice: &signer,) {
        let addr = signer::address_of(alice);
        init_borrow(alice);
        borrow(addr);
    }
}
