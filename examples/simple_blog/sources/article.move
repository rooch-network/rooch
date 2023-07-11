module simple_blog::article {

    use std::error;
    use std::signer;
    use std::string::String; 
    use moveos_std::event;
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_storage;
    use moveos_std::storage_context::{Self, StorageContext};

    const EDATA_TOO_LONG: u64 = 1;
    const ENOT_OWNER_ACCOUNT: u64 = 2;

    struct Article has key {
        version: u64,
        title: String,
        body: String,
    }

    struct ArticleCreatedEvent has key,copy,store {
        id: ObjectID,
    }

    struct ArticleUpdatedEvent has key,copy,store {
        id: ObjectID,
        version: u64,
    }

    struct ArticleDeletedEvent has key,copy,store {
        id: ObjectID,
        version: u64,
    }


    /// Create article
    public fun create_article(
        ctx: &mut StorageContext,
        owner: &signer,
        title: String,
        body: String,
    ): ObjectID {
        assert!(std::string::length(&title) <= 200, error::invalid_argument(EDATA_TOO_LONG));
        assert!(std::string::length(&body) <= 2000, error::invalid_argument(EDATA_TOO_LONG));

        let tx_ctx = storage_context::tx_context_mut(ctx);
        let article = Article {
            version: 0,
            title,
            body,
        };
        let owner_address = signer::address_of(owner);
        let article_obj = object::new(
            tx_ctx,
            owner_address,
            article,
        );
        let id = object::id(&article_obj);
        let object_storage = storage_context::object_storage_mut(ctx);
        object_storage::add(object_storage, article_obj);

        let article_created_event = ArticleCreatedEvent {
            id,
        };
        event::emit_event(ctx, article_created_event);
        id
    }

    /// Update article
    public fun update_article(
        ctx: &mut StorageContext,
        owner: &signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
    ) {
        assert!(std::string::length(&new_title) <= 200, error::invalid_argument(EDATA_TOO_LONG));
        assert!(std::string::length(&new_body) <= 2000, error::invalid_argument(EDATA_TOO_LONG));

        let object_storage = storage_context::object_storage_mut(ctx);
        let article_obj = object_storage::borrow_mut<Article>(object_storage, id);
        let owner_address = signer::address_of(owner);
        
        // only article owner can update the article 
        assert!(object::owner(article_obj) == owner_address, error::permission_denied(ENOT_OWNER_ACCOUNT));

        let article = object::borrow_mut(article_obj);
        article.version = article.version + 1;
        article.title = new_title;
        article.body = new_body;

        let article_update_event = ArticleUpdatedEvent {
            id,
            version: article.version,
        };
        event::emit_event(ctx, article_update_event);
    }

    /// Delete article
    public fun delete_article(
        ctx: &mut StorageContext,
        owner: &signer,
        id: ObjectID,
    ) {
        let object_storage = storage_context::object_storage_mut(ctx);
        let article_obj = object_storage::remove<Article>(object_storage, id);
        let owner_address = signer::address_of(owner);
        
        // only article owner can delete the article 
        assert!(object::owner(&article_obj) == owner_address, error::permission_denied(ENOT_OWNER_ACCOUNT));

        let article_deleted_event = ArticleDeletedEvent {
            id,
            version: object::borrow(&article_obj).version,
        };
        event::emit_event(ctx, article_deleted_event);
        drop_article(article_obj);
    }

    fun drop_article(article_obj: Object<Article>) {
        let (_id, _owner, article) =  object::unpack(article_obj);
        let Article {
            version: _version,
            title: _title,
            body: _body,
        } = article;
    }

    /// Read function of article

    /// get article object by id
    public fun get_article(ctx: &StorageContext, article_id: ObjectID): &Object<Article> {
        let obj_store = storage_context::object_storage(ctx);
        object_storage::borrow(obj_store, article_id)
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
