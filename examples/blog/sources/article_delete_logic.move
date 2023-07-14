module rooch_examples::article_delete_logic {
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;
    use rooch_examples::blog_aggregate;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        article_obj: &Object<article::Article>,
    ): article::ArticleDeleted {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_deleted(
            article_obj,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
        article_deleted: &article::ArticleDeleted,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let _ = article_deleted;
        blog_aggregate::remove_article(storage_ctx, _account, article::id(&article_obj));
        article_obj
    }

}
