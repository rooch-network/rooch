// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::article_create_logic {
    
    use moveos_std::object;
    use moveos_std::object_id::ObjectID;
    use rooch_examples::article;
    use rooch_examples::article_created;
    use std::string::String;
    use rooch_examples::blog_aggregate;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        
        account: &signer,
        title: String,
        body: String,
    ): article::ArticleCreated {
        
        let _ = account;
        article::new_article_created(
            title,
            body,
        )
    }

    public(friend) fun mutate(
        
        _account: &signer,
        article_created: &article::ArticleCreated,
    ) : ObjectID {
        let title = article_created::title(article_created);
        let body = article_created::body(article_created);
        let article_obj = article::create_article(
            
            title,
            body,
        );
        let article_id = object::id(&article_obj);
        blog_aggregate::add_article(article_obj);
        article_id
    }

}
