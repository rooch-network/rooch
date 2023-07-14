module rooch_examples::blog_remove_article_logic {
    use std::vector;

    use moveos_std::object_id::ObjectID;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article_removed_from_blog;
    use rooch_examples::blog;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        article_id: ObjectID,
        blog: &blog::Blog,
    ): blog::ArticleRemovedFromBlog {
        let _ = storage_ctx;
        let _ = account;
        blog::new_article_removed_from_blog(
            blog,
            article_id,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        _account: &signer,
        article_removed_from_blog: &blog::ArticleRemovedFromBlog,
        blog: blog::Blog,
    ): blog::Blog {
        let _ = storage_ctx;
        let article_id = article_removed_from_blog::article_id(article_removed_from_blog);
        let articles = blog::articles(&blog);
        let (found, idx) = vector::index_of(&articles, &article_id);
        if (found) {
            vector::remove(&mut articles, idx);
            blog::set_articles(&mut blog, articles);
        };
        blog
    }
}
