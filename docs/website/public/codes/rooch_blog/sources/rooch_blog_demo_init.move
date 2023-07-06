module rooch_blog::rooch_blog_demo_init {
    use moveos_std::storage_context::StorageContext;
    use rooch_blog::article;

    public entry fun initialize(storage_ctx: &mut StorageContext, account: &signer) {
        article::initialize(storage_ctx, account);
    }

}
