module rooch_examples::blog_create_logic {
    use moveos_std::object_id::ObjectID;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::blog;
    use rooch_examples::blog_created;
    use std::string::String;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        name: String,
        articles: vector<ObjectID>,
    ): blog::BlogCreated {
        let _ = storage_ctx;
        let _ = account;
        blog::new_blog_created(
            name,
            articles,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
        blog_created: &blog::BlogCreated,
    ): blog::Blog {
        let name = blog_created::name(blog_created);
        let articles = blog_created::articles(blog_created);
        let _ = storage_ctx;
        blog::new_blog(
            name,
            articles,
        )
    }

}
