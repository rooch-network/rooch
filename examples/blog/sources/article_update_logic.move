// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::article_update_logic {
    use std::signer;
    use moveos_std::object_ref::{Self, ObjectRef};
    use moveos_std::context::Context;
    use rooch_examples::article::{Self, Article};
    use rooch_examples::article_updated;
    use std::string::String;

    friend rooch_examples::article_aggregate;

    const ErrorNotOwnerAccount: u64 = 113;

    public(friend) fun verify(
        ctx: &mut Context,
        account: &signer,
        title: String,
        body: String,
        article_obj: &ObjectRef<Article>,
    ): article::ArticleUpdated {
        let _ = ctx;
        assert!(signer::address_of(account) == object_ref::owner(article_obj), ErrorNotOwnerAccount);
        article::new_article_updated(
            article_obj,
            title,
            body,
        )
    }

    public(friend) fun mutate(
        ctx: &mut Context,
        _account: &signer,
        article_updated: &article::ArticleUpdated,
        article_obj: ObjectRef<Article>,
    ): ObjectRef<Article> {
        let title = article_updated::title(article_updated);
        let body = article_updated::body(article_updated);
        let id = article::id(&article_obj);
        let _ = ctx;
        let _ = id;
        article::set_title(&mut article_obj, title);
        article::set_body(&mut article_obj, body);
        article_obj
    }

}
