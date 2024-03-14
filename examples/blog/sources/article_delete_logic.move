// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::article_delete_logic {
    use moveos_std::object::Object;
    
    use moveos_std::object_id::ObjectID;
    use rooch_examples::article::{Self, Article};
    use rooch_examples::blog_aggregate;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        account: &signer,
        article_obj: &Object<Article>,
    ): article::ArticleDeleted {
        let _ = account;
        article::new_article_deleted(
            article_obj,
        )
    }

    public(friend) fun mutate(
        
        _account: &signer,
        article_deleted: &article::ArticleDeleted,
        article_id: ObjectID,
    ) : Object<Article> {
        let _ = article_deleted;
        blog_aggregate::remove_article(article_id)
    }

}
