//# init --addresses creator=0x42

//# publish
module creator::test0 {
    struct KeyStruct has key, drop {
        x: u64,
    }
    public fun new_key_struct(x: u64) : KeyStruct {
        KeyStruct { x }
    }
}

//# publish
module creator::test {
    use std::string;
    use creator::test0::{Self, KeyStruct};
    use moveos_std::context::{Self, Context};
    use moveos_std::object;

    struct Foo has key {
        x: u64,
    }

    #[private_generics(T1)]
    public fun publish_foo<T1>(ctx: &mut Context, s: &signer) {
        context::move_resource_to<Foo>(ctx, s, Foo { x: 500 })
    }

    public fun run(ctx: &mut Context, s: &signer) {
        let _ = string::utf8(b"account_storage");
        publish_foo<KeyStruct>(ctx, s)
    }

    public fun call_moveos_std<T:store>(ctx: &mut Context) {
        let object = context::new_object(ctx, test0::new_key_struct(100));
        let _key_struct = object::remove(object);
    }
}
