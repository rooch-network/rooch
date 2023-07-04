module rooch_blog::article_update_logic {
    use std::string::String;

    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_blog::article;
    use rooch_blog::article_updated;

    friend rooch_blog::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
        article_obj: &Object<article::Article>,
    ): article::ArticleUpdated {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_updated(
            article_obj,
            title,
            body,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        article_updated: &article::ArticleUpdated,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let title = article_updated::title(article_updated);
        let body = article_updated::body(article_updated);
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        article::set_title(&mut article_obj, title);
        article::set_body(&mut article_obj, body);
        article_obj
    }
}
