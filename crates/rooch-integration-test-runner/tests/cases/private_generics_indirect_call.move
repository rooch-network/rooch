//# init --addresses creator=0x42

//# publish
module creator::test0 {
    struct KeyStruct has key {
        x: u64,
    }
}

//# publish
module creator::test {
    struct Foo has key {
        x: u64,
    }
    use std::string::{Self};

    #[private_generics(T1)]
    public fun publish_foo<T1>(s: &signer) {
        move_to<Foo>(s, Foo { x: 500 })
    }

    use creator::test0::KeyStruct;
    // use moveos_std::account_storage::AccountStorage;

    public fun run(s: &signer) {
        let _ = string::utf8(b"account_storage");
        publish_foo<Foo>(s)
    }

    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object;
    use moveos_std::object_id::ObjectID;
    use moveos_std::account_storage;
    use moveos_std::object_storage;
    use std::debug;

    struct S has store, key { v: u8 }
    struct Cup<phantom T: store> has store, key { v: u8 }

    public fun call_moveos_std<T:store>(ctx: &mut StorageContext, sender: signer, object_id: ObjectID) {
        let object_storage = storage_context::object_storage_mut(ctx);
        //let obj = object_storage::remove<Cup<S>>(object_storage, object_id);
        let obj = object_storage::remove<KeyStruct>(object_storage, object_id);
        debug::print(&obj);
        let (_id,_owner,value) = object::unpack(obj);
        account_storage::global_move_to(ctx, &sender, value);
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(s: signer) {
        test::run(&s);
    }
}