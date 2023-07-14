module rooch_examples::article_update_logic {
    use std::signer;
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;
    use rooch_examples::article_updated;
    use std::string::String;
    use moveos_std::object;

    friend rooch_examples::article_aggregate;

    const ENOT_OWNER_ACCOUNT: u64 = 113;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
        article_obj: &Object<article::Article>,
    ): article::ArticleUpdated {
        let _ = storage_ctx;
        assert!(signer::address_of(account) == object::owner(article_obj), ENOT_OWNER_ACCOUNT);
        article::new_article_updated(
            article_obj,
            title,
            body,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
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
