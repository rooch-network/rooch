module rooch_examples::article_create_logic {
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;
    use rooch_examples::article_created;
    use std::string::String;
    use rooch_examples::blog_aggregate;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
    ): article::ArticleCreated {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_created(
            title,
            body,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
        article_created: &article::ArticleCreated,
    ): Object<article::Article> {
        let title = article_created::title(article_created);
        let body = article_created::body(article_created);
        let article_obj = article::create_article(
            storage_ctx,
            title,
            body,
        );
        // ///////////////////////////
        blog_aggregate::add_article(storage_ctx, _account, article::id(&article_obj));
        // ///////////////////////////
        article_obj
    }

}
