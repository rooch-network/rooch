// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::blog_remove_article_logic {
    use moveos_std::object::ObjectID;
    use moveos_std::object_ref::ObjectRef;
    use moveos_std::table;
    use rooch_examples::article_removed_from_blog;
    use rooch_examples::blog;
    use rooch_examples::article::Article;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        article_id: ObjectID,
        blog: &blog::Blog,
    ): blog::ArticleRemovedFromBlog {
        blog::new_article_removed_from_blog(
            blog,
            article_id,
        )
    }

    public(friend) fun mutate(
        article_removed_from_blog: &blog::ArticleRemovedFromBlog,
        blog: &mut blog::Blog,
    ) : ObjectRef<Article> {
        let article_id = article_removed_from_blog::article_id(article_removed_from_blog);
        let articles = blog::articles_mut(blog);
        table::remove(articles, article_id)
    }
}
