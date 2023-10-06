//# init --addresses creator=0x42

//# publish
module creator::test {
    use std::string;
    use moveos_std::account_storage;
    use moveos_std::context::{Self, Context};
    use moveos_std::object;
    use moveos_std::object_id::ObjectID;
    use std::debug;

    struct Foo has key, store {
        x: u64,
    }

    #[private_generics(T1)]
    fun publish_foo<T1: store>(ctx: &mut Context, s: &signer) {
        account_storage::global_move_to<Foo>(ctx, s, Foo { x: 500 })
    }

    public fun run(ctx: &mut Context, s: &signer) {
        let _ = string::utf8(b"account_storage");
        publish_foo<Foo>(ctx, s)
    }

    public fun call_moveos_std<T: store>(ctx: &mut Context, sender: &signer, object_id: ObjectID) {
        debug::print(&object_id);
        let obj = context::remove_object<Foo>(ctx, object_id);
        debug::print(&obj);
        let (_id,_owner,value) = object::unpack(obj);
        account_storage::global_move_to<Foo>(ctx, sender, value);
    }
}

//# run --signers creator
script {
    use creator::test;
    use moveos_std::context::Context;

    fun main(ctx: &mut Context, s: &signer) {
        test::run(ctx, s);
    }
}

//# view
//#     --address 0x42
//#     --resource 0x42::test::Foo
