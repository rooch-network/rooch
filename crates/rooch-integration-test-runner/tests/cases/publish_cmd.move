//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::account_storage;
    struct Foo has key {
        x: u64,
    }

    public fun publish_foo(ctx: &mut StorageContext, s: &signer) {
        account_storage::global_move_to<Foo>(ctx, s, Foo { x: 500 })
    }
}

//# run --signers creator
script {
    use creator::test;
    use moveos_std::storage_context::{StorageContext};

    fun main(ctx: &mut StorageContext, s: signer) {
        test::publish_foo(ctx, &s);
    }
}

//# view
//#     --address 0x42
//#     --resource 0x42::test::Foo