// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Name the module to `simple_blog` for avoid name conflict with `examples/blog`
module simple_blog::simple_blog {
    use std::signer;
    use std::string::{Self, String};
    use std::vector;

    use moveos_std::account;
    use moveos_std::object::{Self, Object, ObjectID};

    use simple_blog::simple_article::{Self, Article};

    const ErrorDataTooLong: u64 = 1;
    const ErrorNotFound: u64 = 2;

    struct MyBlog has key, store {
        name: String,
        articles: vector<ObjectID>,
    }

    /// This init function is called when the module is published
    /// The owner is the address of the account that publishes the module
    fun init(owner: &signer) {
        // auto create blog for module publisher
        create_blog(owner);
    }

    public fun create_blog(owner: &signer) {
        let articles = vector::empty();
        let myblog = MyBlog {
            name: string::utf8(b"MyBlog"),
            articles,
        };
        account::move_resource_to(owner, myblog);
    }

    public entry fun set_blog_name(owner: &signer, blog_name: String) {
        assert!(std::string::length(&blog_name) <= 200, ErrorDataTooLong);
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if (!account::exists_resource<MyBlog>(owner_address)) {
            create_blog(owner);
        };
        let myblog = account::borrow_mut_resource<MyBlog>(owner_address);
        myblog.name = blog_name;
    }

    /// Get owner's blog's articles
    public fun get_blog_articles(owner_address: address): &vector<ObjectID> {
        let myblog = account::borrow_resource<MyBlog>(owner_address);
        &myblog.articles
    }

    fun add_article_to_myblog(owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if (!account::exists_resource<MyBlog>(owner_address)) {
            create_blog(owner);
        };
        let myblog = account::borrow_mut_resource<MyBlog>(owner_address);
        vector::push_back(&mut myblog.articles, article_id);
    }

    public entry fun create_article(
        owner: signer,
        title: String,
        body: String,
    ) {
        let article_id = simple_article::create_article(&owner, title, body);
        add_article_to_myblog(&owner, article_id);
    }

    public entry fun update_article(
        article_obj: &mut Object<Article>,
        new_title: String,
        new_body: String,
    ) {
        simple_article::update_article(article_obj, new_title, new_body);
    }

    fun delete_article_from_myblog(owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        let myblog = account::borrow_mut_resource<MyBlog>(owner_address);
        let (contains, index) = vector::index_of(&myblog.articles, &article_id);
        assert!(contains, ErrorNotFound);
        vector::remove(&mut myblog.articles, index);
    }

    public entry fun delete_article(
        owner: &signer,
        article_id: ObjectID,
    ) {
        delete_article_from_myblog(owner, article_id);
        let article_obj = object::take_object(owner, article_id);
        simple_article::delete_article(article_obj);
    }
}