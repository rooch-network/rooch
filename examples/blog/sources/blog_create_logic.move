// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::blog_create_logic {
    use moveos_std::object_id::ObjectID;
    use moveos_std::context::Context;
    use rooch_examples::blog;
    use rooch_examples::blog_created;
    use std::string::String;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        ctx: &mut Context,
        account: &signer,
        name: String,
        articles: vector<ObjectID>,
    ): blog::BlogCreated {
        let _ = ctx;
        let _ = account;
        blog::new_blog_created(
            name,
            articles,
        )
    }

    public(friend) fun mutate(
        ctx: &mut Context,
        _account: &signer,
        blog_created: &blog::BlogCreated,
    ): blog::Blog {
        let name = blog_created::name(blog_created);
        let articles = blog_created::articles(blog_created);
        let _ = ctx;
        blog::new_blog(
            name,
            articles,
        )
    }

}
