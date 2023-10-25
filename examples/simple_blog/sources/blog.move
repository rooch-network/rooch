// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module simple_blog::blog {
    use std::error;
    use std::signer;
    use std::string::{Self,String};
    use std::vector;
    use moveos_std::object::ObjectID;
    use moveos_std::object_ref::{Self, ObjectRef};
    use moveos_std::context::Context;
    use moveos_std::account_storage;
    use simple_blog::article::{Self, Article};

    const ErrorDataTooLong: u64 = 1;
    const ErrorNotFound: u64 = 2;

    struct MyBlog has key {
        name: String,
        articles: vector<ObjectRef<Article>>,
    }

    /// This init function is called when the module is published
    /// The owner is the address of the account that publishes the module
    fun init(ctx: &mut Context, owner: &signer) {
        // auto create blog for module publisher 
        create_blog(ctx, owner);
    }

    public fun create_blog(ctx: &mut Context, owner: &signer) {
        let articles = vector::empty();
        let myblog = MyBlog{
            name: string::utf8(b"MyBlog"),
            articles,
        };
        account_storage::global_move_to(ctx, owner, myblog);
    }

    public entry fun set_blog_name(ctx: &mut Context, owner: &signer, blog_name: String) {
        assert!(std::string::length(&blog_name) <= 200, error::invalid_argument(ErrorDataTooLong));
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            create_blog(ctx, owner);
        };
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        myblog.name = blog_name;
    }

    fun add_article_to_myblog(ctx: &mut Context, owner: &signer, article_obj: ObjectRef<Article>) {
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            create_blog(ctx, owner);
        };
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        vector::push_back(&mut myblog.articles, article_obj);
    }

    fun delete_article_from_myblog(ctx: &mut Context, owner: &signer, article_id: ObjectID): ObjectRef<Article> {
        let owner_address = signer::address_of(owner);
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        let idx = 0;
        while(idx < vector::length(&myblog.articles)){
            let article_obj = vector::borrow(&myblog.articles, idx);
            if(object_ref::id(article_obj) == article_id){
                return vector::remove(&mut myblog.articles, idx)
            };
            idx = idx + 1;
        };
        abort error::not_found(ErrorNotFound)
    }

    /// Get owner's blog's articles
    public fun get_blog_articles(ctx: &Context, owner_address: address): &vector<ObjectRef<Article>> {
        let myblog = account_storage::global_borrow<MyBlog>(ctx, owner_address);
        &myblog.articles
    }

    public entry fun create_article(
        ctx: &mut Context,
        owner: signer,
        title: String,
        body: String,
    ) {
        let article_id = article::create_article(ctx, &owner, title, body);
        add_article_to_myblog(ctx, &owner, article_id);
    }

    public entry fun update_article(
        ctx: &mut Context,
        article_obj: &mut ObjectRef<Article>,
        new_title: String,
        new_body: String,
    ) {
        article::update_article(ctx, article_obj, new_title, new_body);
    }

    public entry fun delete_article(
        ctx: &mut Context,
        owner: &signer,
        article_id: ObjectID,
    ) {
        let article_obj = delete_article_from_myblog(ctx, owner, article_id);
        article::delete_article(ctx, article_obj);
    }
}
