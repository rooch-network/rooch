// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Name the module to `simple_article` for avoid name conflict with `examples/blog`
module simple_blog::simple_article {

    use std::error;
    use std::signer;
    use std::string::String;
    use moveos_std::event;
    use moveos_std::object::{ObjectID};
    use moveos_std::object::{Self, Object};
    use moveos_std::context::{Self, Context};

    const ErrorDataTooLong: u64 = 1;
    const ErrorNotOwnerAccount: u64 = 2;

    //TODO should we allow Article to be transferred?
    struct Article has key, store {
        version: u64,
        title: String,
        body: String,
    }

    struct ArticleCreatedEvent has copy, store, drop {
        id: ObjectID,
    }

    struct ArticleUpdatedEvent has copy, store, drop {
        id: ObjectID,
        version: u64,
    }

    struct ArticleDeletedEvent has copy, store, drop {
        id: ObjectID,
        version: u64,
    }


    /// Create article
    public fun create_article(
        ctx: &mut Context,
        owner: &signer,
        title: String,
        body: String,
    ): ObjectID {
        assert!(std::string::length(&title) <= 200, error::invalid_argument(ErrorDataTooLong));
        assert!(std::string::length(&body) <= 2000, error::invalid_argument(ErrorDataTooLong));

        let article = Article {
            version: 0,
            title,
            body,
        };
        let owner_addr = signer::address_of(owner);
        let article_obj = context::new_object(
            ctx,
            article,
        );
        let id = object::id(&article_obj);

        let article_created_event = ArticleCreatedEvent {
            id,
        };
        event::emit(article_created_event);
        object::transfer(article_obj, owner_addr);
        id
    }

    /// Update article
    public fun update_article(
        article_obj: &mut Object<Article>,
        new_title: String,
        new_body: String,
    ) {
        assert!(std::string::length(&new_title) <= 200, error::invalid_argument(ErrorDataTooLong));
        assert!(std::string::length(&new_body) <= 2000, error::invalid_argument(ErrorDataTooLong));

        let id = object::id(article_obj);
        let article = object::borrow_mut(article_obj);
        article.version = article.version + 1;
        article.title = new_title;
        article.body = new_body;

        let article_update_event = ArticleUpdatedEvent {
            id,
            version: article.version,
        };
        event::emit(article_update_event);
    }

    /// Delete article
    public fun delete_article(
        article_obj: Object<Article>,
    ) {
        let id = object::id(&article_obj);
        let article = object::remove(article_obj);

        let article_deleted_event = ArticleDeletedEvent {
            id,
            version: article.version,
        };
        event::emit(article_deleted_event);
        drop_article(article);
    }

    fun drop_article(article: Article) {
        let Article {
            version: _version,
            title: _title,
            body: _body,
        } = article;
    }

    /// Read function of article


    /// get article version
    public fun version(article: &Article): u64 {
        article.version
    }

    /// get article title
    public fun title(article: &Article): String {
        article.title
    }

    /// get article body
    public fun body(article: &Article): String {
        article.body
    }
}
