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
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let sender = tx_context::sender(tx_ctx);
        let tx_hash = tx_context::tx_hash(tx_ctx);
        debug::print(&tx_hash);
        // if the tx hash change, need to figure out why.
        assert!(x"7852c5dcbd87e82102dba0db36d44b5a9fb0006b3e828c0b5f0832f70a8ff6ee" == tx_hash, 1000);
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

//# view_object --object-id 0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647

// Mint Cup<S> to A.

//# run test::m::mint_cup --type-args test::m::S --signers A

//# view_object --object-id 0x0bbaf311ae6768a532b1f9dee65b1758a7bb1114fd57df8fa94cb2d1cb5f6896

// Move S to global.
//Currently, we use @address to pass object argument to the transaction, define a new way to pass object argument to the transaction.

//# run test::m::move_s_to_global --signers A --args @0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647

//# view --address A --resource test::m::S

// Move Cup<S> to global.

//# run test::m::move_cup_to_global --signers A  --type-args test::m::S --args @0x0bbaf311ae6768a532b1f9dee65b1758a7bb1114fd57df8fa94cb2d1cb5f6896

//# view --address A --resource test::m::Cup<test::m::S>
