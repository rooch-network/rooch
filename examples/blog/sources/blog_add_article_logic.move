// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::blog_add_article_logic {

    use moveos_std::object::Object;
    use moveos_std::object_id::ObjectID;
    use moveos_std::table;
    use rooch_examples::article_added_to_blog;
    use rooch_examples::blog;
    use rooch_examples::article::Article;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        article_id: ObjectID,
        blog: &blog::Blog,
    ): blog::ArticleAddedToBlog {
        blog::new_article_added_to_blog(
            blog,
            article_id,
        )
    }

    public(friend) fun mutate(
        article_added_to_blog: &blog::ArticleAddedToBlog,
        article_obj: Object<Article>,
        blog: &mut blog::Blog,
    ) {
        let article_id = article_added_to_blog::article_id(article_added_to_blog);
        let articles = blog::articles_mut(blog);
        table::add(articles, article_id, article_obj);
    }
}
