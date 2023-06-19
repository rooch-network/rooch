//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::account_storage;
    struct Foo has key {
        x: u64,
    }

    #[private(T1)]
    public fun publish_foo<T1>(ctx: &mut StorageContext, s: &signer) {
        account_storage::global_move_to<Foo>(ctx, s, Foo { x: 500 })
    }

    public fun run(ctx: &mut StorageContext, s: &signer) {
        publish_foo<u64>(ctx, s)
    }
}

//# run --signers creator
script {
    use creator::test;
    use moveos_std::storage_context::{StorageContext};

    fun main(ctx: &mut StorageContext, s: signer) {
        test::run(ctx, &s);
    }
}