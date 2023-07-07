module rooch_blog::rooch_blog {
    use std::error;
    use std::signer;
    use std::string::String;
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_blog::article;

    const EID_DATA_TOO_LONG: u64 = 102;
    const EINAPPROPRIATE_VERSION: u64 = 103;
    const ENOT_GENESIS_ACCOUNT: u64 = 105;

    // === Initialize ===

    // Define a function that initialize the blog
    fun init_blog(storage_ctx: &mut StorageContext, account: &signer) {
        assert!(signer::address_of(account) == @rooch_blog, error::invalid_argument(ENOT_GENESIS_ACCOUNT));
        let _ = storage_ctx;
        let _ = account;
    }

    // The entry function that initializes.
    entry fun initialize(storage_ctx: &mut StorageContext, account: &signer) {
        init_blog(storage_ctx, account);
    }

    // === Create ===

    fun create_verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
    ): article::ArticleCreated {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_created(
            title,
            body,
        )
    }

    fun create_mutate(
        storage_ctx: &mut StorageContext,
        article_created: &article::ArticleCreated,
    ): Object<article::Article> {
        let title = article::article_created_title(article_created);
        let body = article::article_created_body(article_created);
        article::create_article(
            storage_ctx,
            title,
            body,
        )
    }

    public entry fun create(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
    ) {
        let article_created = create_verify(
            storage_ctx,
            account,
            title,
            body,
        );
        let article_obj = create_mutate(
            storage_ctx,
            &article_created,
        );
        article::set_article_created_id(&mut article_created, article::id(&article_obj));
        article::add_article(storage_ctx, article_obj);
        article::emit_article_created(storage_ctx, article_created);
    }

    // === Update ===

    fun update_verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
        article_obj: &Object<article::Article>,
    ): article::ArticleUpdated {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_updated(
            article_obj,
            title,
            body,
        )
    }

    fun update_mutate(
        storage_ctx: &mut StorageContext,
        article_updated: &article::ArticleUpdated,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let title = article::article_updated_title(article_updated);
        let body = article::article_updated_body(article_updated);
        let id = article::article_updated_id(article_updated);
        let _ = storage_ctx;
        let _ = id;
        article::set_title(&mut article_obj, title);
        article::set_body(&mut article_obj, body);
        article_obj
    }

    public entry fun update(
        storage_ctx: &mut StorageContext,
        account: &signer,
        id: ObjectID,
        title: String,
        body: String,
    ) {
        let article_obj = article::remove_article(storage_ctx, id);
        let article_updated = update_verify(
            storage_ctx,
            account,
            title,
            body,
            &article_obj,
        );
        let updated_article_obj = update_mutate(
            storage_ctx,
            &article_updated,
            article_obj,
        );
        article::update_version_and_add(storage_ctx, updated_article_obj);
        article::emit_article_updated(storage_ctx, article_updated);
    }

    // === Delete ===

    fun delete_verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        article_obj: &Object<article::Article>,
    ): article::ArticleDeleted {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_deleted(
            article_obj,
        )
    }

    fun delete_mutate(
        storage_ctx: &mut StorageContext,
        article_deleted: &article::ArticleDeleted,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        let _ = article_deleted;
        article_obj
    }

    public entry fun delete(
        storage_ctx: &mut StorageContext,
        account: &signer,
        id: ObjectID,
    ) {
        let article_obj = article::remove_article(storage_ctx, id);
        let article_deleted = delete_verify(
            storage_ctx,
            account,
            &article_obj,
        );
        let updated_article_obj = delete_mutate(
            storage_ctx,
            &article_deleted,
            article_obj,
        );
        article::drop_article(updated_article_obj);
        article::emit_article_deleted(storage_ctx, article_deleted);
    }
}
