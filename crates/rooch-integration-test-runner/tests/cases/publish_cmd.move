//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::context::{Context};
    use moveos_std::account_storage;
    struct Foo has key {
        x: u64,
    }

    public fun publish_foo(ctx: &mut Context, s: &signer) {
        account_storage::global_move_to<Foo>(ctx, s, Foo { x: 500 })
    }
}

//# run --signers creator
script {
    use creator::test;
    use moveos_std::context::{Context};

    fun main(ctx: &mut Context, s: signer) {
        test::publish_foo(ctx, &s);
    }
}

//# view
//#     --address 0x42
//#     --resource 0x42::test::Foo