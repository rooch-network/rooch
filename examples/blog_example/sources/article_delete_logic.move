module rooch_examples::article_delete_logic {
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;

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
        article_deleted: &article::ArticleDeleted,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        let _ = article_deleted;
        article_obj
    }
}
