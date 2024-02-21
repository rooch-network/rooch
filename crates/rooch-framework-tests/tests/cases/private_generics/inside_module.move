//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::context::{Self, Context};
    use moveos_std::account;

    struct Foo has key {
        x: u64,
    }

    #[private_generics(T)]
    fun publish_foo<T: key>(ctx: &mut Context, s: &signer) {
        account::move_resource_to<Foo>(ctx, s, Foo { x: 500})
    }

    public fun invoke_publish_foo(ctx: &mut Context, s: &signer) {
        publish_foo<Foo>(ctx, s);
    }
}

//# run --signers creator
script {
    use moveos_std::context::Context;
    use creator::test;

    fun main(ctx: &mut Context, s: signer) {
        test::invoke_publish_foo(ctx, &s);
    }
}
