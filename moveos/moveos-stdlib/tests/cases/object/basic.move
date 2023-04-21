//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object::{Self, ObjectID};
    use std::debug;

    struct S has store, key { v: u8 }
    struct Cup<phantom T: store> has store, key { v: u8 }

    public entry fun mint_s(ctx: &mut TxContext) {
        let sender = tx_context::sender(ctx);
        let obj = object::new(ctx, sender , S { v: 1});
        debug::print(&obj);
        let object_store = object::load_object_store(ctx);
        object::add(&mut object_store, obj);
    }

    public entry fun move_s_to_global(ctx: &mut TxContext, sender: signer, object_id: ObjectID) {
        let object_store = object::load_object_store(ctx);
        let obj = object::remove<S>(&mut object_store, object_id);
        debug::print(&obj);
        let (_id,value,_owner) = object::unpack(obj);
        move_to(&sender, value);
    }

    public entry fun mint_cup<T: store>(ctx: &mut TxContext) {
        let sender = tx_context::sender(ctx);
        let obj = object::new(ctx, sender, Cup<T> { v: 2 });
        debug::print(&obj);
        let object_store = object::load_object_store(ctx);
        object::add(&mut object_store, obj);
    }

    public entry fun move_cup_to_global<T:store>(ctx: &mut TxContext, sender: signer, object_id: ObjectID) {
        let object_store = object::load_object_store(ctx);
        let obj = object::remove<Cup<T>>(&mut object_store, object_id);
        debug::print(&obj);
        let (_id,value,_owner) = object::unpack(obj);
        move_to(&sender, value);
    }
}

// Mint S to A.

//# run test::m::mint_s --signers A

//# view_object --object-id 0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647

// Mint Cup<S> to A.

//# run test::m::mint_cup --type-args test::m::S --signers A

//# view_object --object-id 0xbbaf311ae6768a532b1f9dee65b1758a7bb1114fd57df8fa94cb2d1cb5f6896

// Move S to global.
//Currently, we use @address to pass object argument to the transaction, define a new way to pass object argument to the transaction.

//# run test::m::move_s_to_global --signers A --args @0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647

//# view --address A --resource test::m::S

// Move Cup<S> to global.

//# run test::m::move_cup_to_global --signers A  --type-args test::m::S --args @0xbbaf311ae6768a532b1f9dee65b1758a7bb1114fd57df8fa94cb2d1cb5f6896

//# view --address A --resource test::m::Cup<test::m::S>
