//# init --addresses creator=0x42

//# publish
module creator::test {
    use std::string;
    use moveos_std::context::{Self, Context};

    struct Foo has key, store {
        x: u64,
    }

    #[private_generics(T1)]
    fun publish_foo<T1: store>(ctx: &mut Context, s: &signer) {
        account::move_resource_to<Foo>(ctx, s, Foo { x: 500 })
    }

    public fun run(ctx: &mut Context, s: &signer) {
        let _ = string::utf8(b"resource_object");
        publish_foo<Foo>(ctx, s)
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
