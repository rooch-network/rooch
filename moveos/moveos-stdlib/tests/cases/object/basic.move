//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    use moveos_std::tx_context;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::account_storage;
    use moveos_std::object_storage;
    use std::debug;

    struct S has store, key { v: u8 }
    struct Cup<phantom T: store> has store, key { v: u8 }

    public entry fun mint_s(ctx: &mut StorageContext) {
        //TODO move to account create
        //account_storage::create_account_storage(ctx, @0x43);

        let tx_ctx = storage_context::tx_context_mut(ctx);
        let sender = tx_context::sender(tx_ctx);
        let obj = object::new(tx_ctx, sender , S { v: 1});
        debug::print(&obj);
        let object_storage = storage_context::object_storage_mut(ctx);
        object_storage::add(object_storage, obj);
    }

    public entry fun move_s_to_global(ctx: &mut StorageContext, sender: signer, object_id: ObjectID) {
        let object_storage = storage_context::object_storage_mut(ctx);
        debug::print(&object_id);
        let obj = object_storage::remove<S>(object_storage, object_id);
        debug::print(&obj);
        let (_id, _owner, value) = object::unpack(obj);
        account_storage::global_move_to(ctx, &sender, value);
    }

    public entry fun mint_cup<T: store>(ctx: &mut StorageContext) {
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let sender = tx_context::sender(tx_ctx);
        let obj = object::new(tx_ctx, sender, Cup<T> { v: 2 });
        debug::print(&obj);
        let object_storage = storage_context::object_storage_mut(ctx);
        object_storage::add(object_storage, obj);
    }

    public entry fun move_cup_to_global<T:store>(ctx: &mut StorageContext, sender: signer, object_id: ObjectID) {
        let object_storage = storage_context::object_storage_mut(ctx);
        let obj = object_storage::remove<Cup<S>>(object_storage, object_id);
        debug::print(&obj);
        let (_id,_owner,value) = object::unpack(obj);
        account_storage::global_move_to(ctx, &sender, value);
    }
}

// Mint S to A.

//# run test::m::mint_s --signers A

//# view_object --object-id 0xa5dac25e36ef3fdb7f496b6ab0d1916d73d025dc1c5f2560f779e62b645cac7d

// Mint Cup<S> to A.

//# run test::m::mint_cup --type-args test::m::S --signers A

//# view_object --object-id 0xbe6975de71303c7a4ab6d2d15b14cb4320fba263cc4283cfe1d63a633247db1

// Move S to global.
//Currently, we use @address to pass object argument to the transaction, define a new way to pass object argument to the transaction.

//# run test::m::move_s_to_global --signers A --args @0xa5dac25e36ef3fdb7f496b6ab0d1916d73d025dc1c5f2560f779e62b645cac7d

//# view --address A --resource test::m::S

// Move Cup<S> to global.

//# run test::m::move_cup_to_global --signers A  --type-args test::m::S --args @0xbe6975de71303c7a4ab6d2d15b14cb4320fba263cc4283cfe1d63a633247db1

//# view --address A --resource test::m::Cup<test::m::S>
