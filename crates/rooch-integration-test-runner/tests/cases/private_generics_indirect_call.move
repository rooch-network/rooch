//# init --addresses creator=0x42

//# publish
module creator::test0 {
    struct KeyStruct has key {
        x: u64,
    }
}

//# publish
module creator::test {
    use std::string;
    use std::debug;
    use creator::test0::KeyStruct;
    use moveos_std::account_storage;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::ObjectID;

    struct Foo has key {
        x: u64,
    }

    #[private_generics(T1)]
    public fun publish_foo<T1>(ctx: &mut Context, s: &signer) {
        account_storage::global_move_to<Foo>(ctx, s, Foo { x: 500 })
    }

    public fun run(ctx: &mut Context, s: &signer) {
        let _ = string::utf8(b"account_storage");
        publish_foo<KeyStruct>(ctx, s)
    }

    public fun call_moveos_std<T:store>(ctx: &mut Context, sender: &signer, object_id: ObjectID) {
        debug::print(&object_id);
        let (_id,_owner,value) = context::remove_object<KeyStruct>(ctx, object_id);
        account_storage::global_move_to(ctx, sender, value);
    }
}
