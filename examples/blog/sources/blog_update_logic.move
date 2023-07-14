module rooch_examples::blog_update_logic {
    use moveos_std::object_id::ObjectID;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::blog;
    use rooch_examples::blog_updated;
    use std::string::String;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        name: String,
        articles: vector<ObjectID>,
        blog: &blog::Blog,
    ): blog::BlogUpdated {
        let _ = storage_ctx;
        let _ = account;
        blog::new_blog_updated(
            blog,
            name,
            articles,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
        blog_updated: &blog::BlogUpdated,
        blog: blog::Blog,
    ): blog::Blog {
        let name = blog_updated::name(blog_updated);
        let articles = blog_updated::articles(blog_updated);
        let _ = storage_ctx;
        blog::set_name(&mut blog, name);
        blog::set_articles(&mut blog, articles);
        blog
    }

}
