module rooch_examples::blog_delete_logic {
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::blog;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        blog: &blog::Blog,
    ): blog::BlogDeleted {
        let _ = storage_ctx;
        let _ = account;
        blog::new_blog_deleted(
            blog,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
        blog_deleted: &blog::BlogDeleted,
        blog: blog::Blog,
    ): blog::Blog {
        let _ = storage_ctx;
        let _ = blog_deleted;
        blog
    }

}
