// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Name the module to `simple_blog` for avoid name conflict with `examples/blog`
module simple_blog::simple_blog {
    use std::signer;
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Object};
    use moveos_std::context::{Self, Context};
    use simple_blog::simple_article::{Self, Article};

    const ErrorDataTooLong: u64 = 1;
    const ErrorNotFound: u64 = 2;

    struct MyBlog has key {
        name: String,
        articles: vector<ObjectID>,
    }

    /// This init function is called when the module is published
    /// The owner is the address of the account that publishes the module
    fun init(ctx: &mut Context, owner: &signer) {
        // auto create blog for module publisher 
        create_blog(ctx, owner);
    }

    public fun create_blog(ctx: &mut Context, owner: &signer) {
        let articles = vector::empty();
        let myblog = MyBlog {
            name: string::utf8(b"MyBlog"),
            articles,
        };
        context::move_resource_to(ctx, owner, myblog);
    }

    public entry fun set_blog_name(ctx: &mut Context, owner: &signer, blog_name: String) {
        assert!(std::string::length(&blog_name) <= 200, ErrorDataTooLong);
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if (!context::exists_resource<MyBlog>(ctx, owner_address)) {
            create_blog(ctx, owner);
        };
        let myblog = context::borrow_mut_resource<MyBlog>(ctx, owner_address);
        myblog.name = blog_name;
    }

    /// Get owner's blog's articles
    public fun get_blog_articles(ctx: &Context, owner_address: address): &vector<ObjectID> {
        let myblog = context::borrow_resource<MyBlog>(ctx, owner_address);
        &myblog.articles
    }

    fun add_article_to_myblog(ctx: &mut Context, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if (!context::exists_resource<MyBlog>(ctx, owner_address)) {
            create_blog(ctx, owner);
        };
        let myblog = context::borrow_mut_resource<MyBlog>(ctx, owner_address);
        vector::push_back(&mut myblog.articles, article_id);
    }

    public entry fun create_article(
        ctx: &mut Context,
        owner: signer,
        title: String,
        body: String,
    ) {
        let article_id = simple_article::create_article(ctx, &owner, title, body);
        add_article_to_myblog(ctx, &owner, article_id);
    }

    public entry fun update_article(
        article_obj: &mut Object<Article>,
        new_title: String,
        new_body: String,
    ) {
        simple_article::update_article(article_obj, new_title, new_body);
    }

    fun delete_article_from_myblog(ctx: &mut Context, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        let myblog = context::borrow_mut_resource<MyBlog>(ctx, owner_address);
        let (contains, index) = vector::index_of(&myblog.articles, &article_id);
        assert!(contains, ErrorNotFound);
        vector::remove(&mut myblog.articles, index);
    }

    public entry fun delete_article(
        ctx: &mut Context,
        owner: &signer,
        article_id: ObjectID,
    ) {
        delete_article_from_myblog(ctx, owner, article_id);
        let article_obj = context::take_object(ctx, owner, article_id);
        simple_article::delete_article(article_obj);
    }
}
