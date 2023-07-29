//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::storage_context::StorageContext;
    use moveos_std::account_storage;

    struct Foo has key {
        x: u64,
    }

    #[private_generics(T)]
    fun publish_foo<T: key>(ctx: &mut StorageContext, s: &signer) {
        account_storage::global_move_to<Foo>(ctx, s, Foo { x: 500})
    }

    public fun invoke_publish_foo(ctx: &mut StorageContext, s: &signer) {
        publish_foo<Foo>(ctx, s);
    }
}

//# run --signers creator
script {
    use moveos_std::storage_context::StorageContext;
    use creator::test;

    fun main(ctx: &mut StorageContext, s: signer) {
        test::invoke_publish_foo(ctx, &s);
    }
}
