module rooch_examples::article_update_logic {
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;
    use rooch_examples::article_updated;
    use std::string::String;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
        owner: address,
        article_obj: &Object<article::Article>,
    ): article::ArticleUpdated {
        let _ = storage_ctx;
        let _ = account;
        assert!(std::signer::address_of(account) == article::owner(article_obj), 111);
        article::new_article_updated(
            article_obj,
            title,
            body,
            owner,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        article_updated: &article::ArticleUpdated,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let title = article_updated::title(article_updated);
        let body = article_updated::body(article_updated);
        let owner = article_updated::owner(article_updated);
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        article::set_title(&mut article_obj, title);
        article::set_body(&mut article_obj, body);
        article::set_owner(&mut article_obj, owner);
        article_obj
    }

}
