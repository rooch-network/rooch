// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module simple_blog::article {

    use std::error;
    use std::signer;
    use std::string::String; 
    use moveos_std::event;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::object_ref;
    use moveos_std::context::{Self, Context};

    const ErrorDataTooLong: u64 = 1;
    const ErrorNotOwnerAccount: u64 = 2;

    struct Article has key {
        version: u64,
        title: String,
        body: String,
    }

    struct ArticleCreatedEvent has copy,store {
        id: ObjectID,
    }

    struct ArticleUpdatedEvent has copy,store {
        id: ObjectID,
        version: u64,
    }

    struct ArticleDeletedEvent has copy,store {
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
        let article_ref = context::new_object_with_owner(
            ctx,
            owner_addr,
            article,
        );
        let id = object_ref::id(&article_ref);

        let article_created_event = ArticleCreatedEvent {
            id,
        };
        event::emit(ctx, article_created_event);
        id
    }

    /// Update article
    public fun update_article(
        ctx: &mut Context,
        owner: &signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
    ) {
        assert!(std::string::length(&new_title) <= 200, error::invalid_argument(ErrorDataTooLong));
        assert!(std::string::length(&new_body) <= 2000, error::invalid_argument(ErrorDataTooLong));

        let article_obj = context::borrow_object_mut<Article>(ctx, id);
        let owner_address = signer::address_of(owner);
        
        // only article owner can update the article 
        assert!(object::owner(article_obj) == owner_address, error::permission_denied(ErrorNotOwnerAccount));

        let article = object::borrow_mut(article_obj);
        article.version = article.version + 1;
        article.title = new_title;
        article.body = new_body;

        let article_update_event = ArticleUpdatedEvent {
            id,
            version: article.version,
        };
        event::emit(ctx, article_update_event);
    }

    /// Delete article
    public fun delete_article(
        ctx: &mut Context,
        owner: &signer,
        id: ObjectID,
    ) {

        
        let owner_address = signer::address_of(owner);
        let (id, owner, article) = context::remove_object<Article>(ctx, id);
        
        // only article owner can delete the article 
        assert!(owner == owner_address, error::permission_denied(ErrorNotOwnerAccount));

        let article_deleted_event = ArticleDeletedEvent {
            id,
            version: article.version,
        };
        event::emit(ctx, article_deleted_event);
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

    /// get article object by id
    public fun get_article(ctx: &Context, article_id: ObjectID): &Object<Article> {
        context::borrow_object<Article>(ctx, article_id)
    }

    /// get article id
    public fun id(article_obj: &Object<Article>): ObjectID {
        object::id(article_obj)
    }

    /// get article version
    public fun version(article_obj: &Object<Article>): u64 {
        object::borrow(article_obj).version
    }

    /// get article title
    public fun title(article_obj: &Object<Article>): String {
        object::borrow(article_obj).title
    }

    /// get article body
    public fun body(article_obj: &Object<Article>): String {
        object::borrow(article_obj).body
    }
    
}
