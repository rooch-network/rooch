// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::borrow_test {
    use rooch_examples::borrowd::{BorrowCapability, DataStore};
    use rooch_examples::borrowd;
    use moveos_std::account_storage;
    use moveos_std::context::{Context};

    #[test_only]
    use std::signer;
    #[test_only]
    use moveos_std::context;

    struct Capabilities has key {
        borrow_cap: BorrowCapability,
    }

    struct DataStoreWrapper has key {
        data_store: DataStore,
    }

    public fun init_borrow(
        ctx: &mut Context,
        account: &signer,
    ) {
        let borrow_cap = borrowd::new_borrow_cap();
        let data_store = borrowd::new_data_store();
        account_storage::global_move_to<Capabilities>(ctx, account, Capabilities {
            borrow_cap,
        });
        account_storage::global_move_to<DataStoreWrapper>(ctx, account, DataStoreWrapper {
            data_store,
        });
    }

    public fun borrow(
        ctx: &mut Context,
        addr: address,
    ) {
        let cap = account_storage::global_borrow_mut<Capabilities>(ctx, addr);
        borrowd::do_immutable_borrow(ctx, &cap.borrow_cap);

        //  Invalid usage of reference as function argument. Cannot transfer a mutable reference that is being borrowed
        // let data_store_warpper = account_storage::global_borrow_mut<DataStoreWrapper>(ctx, addr);
        // borrowd::do_mutable_borrow(ctx, addr, &mut data_store_warpper.data_store);
    }

    #[test(alice = @0x11)]
    fun test_borrow(alice: &signer,) {
        let addr = signer::address_of(alice);
        let ctx = context::new_test_context(addr);
        init_borrow(&mut ctx, alice);
        borrow(&mut ctx, addr);
        moveos_std::context::drop_test_context(ctx);
    }
}
