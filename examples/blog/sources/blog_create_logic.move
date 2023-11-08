// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::blog_create_logic {
    use moveos_std::object::ObjectID;
    use moveos_std::object::Object;
    use moveos_std::context::{Self,Context};
    use rooch_examples::blog;
    use rooch_examples::blog_created;
    use rooch_examples::article::Article;
    use std::string::String;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        ctx: &mut Context,
        account: &signer,
        name: String,
    ): blog::BlogCreated {
        let _ = ctx;
        let _ = account;
        blog::new_blog_created(
            name,
        )
    }

    public(friend) fun mutate(
        ctx: &mut Context,
        _account: &signer,
        blog_created: &blog::BlogCreated,
    ): blog::Blog {
        let name = blog_created::name(blog_created);
        let articles = context::new_table<ObjectID, Object<Article>>(ctx);
        blog::new_blog(
            name,
            articles,
        )
    }

}
