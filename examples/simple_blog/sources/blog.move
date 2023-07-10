module rooch_examples::rooch_blog {
    use std::error;
    use std::signer;
    use std::string::{Self,String};
    use std::vector;
    use moveos_std::object_id::ObjectID;
    use moveos_std::storage_context::StorageContext;
    use moveos_std::account_storage;
    use rooch_examples::article;

    const EDATA_TOO_LONG: u64 = 1;
    const ENOT_FOUND: u64 = 2;

    struct MyBlog has key {
        name: String,
        articles: vector<ObjectID>,
    }

    public entry fun set_blog_title(ctx: &mut StorageContext, owner: &signer, blog_name: String) {
        assert!(std::string::length(&blog_name) <= 200, error::invalid_argument(EDATA_TOO_LONG));
        let owner_address = signer::address_of(owner);
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        myblog.name = blog_name;
    }

    fun add_article_to_myblog(ctx: &mut StorageContext, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        if(account_storage::global_exists<MyBlog>(ctx, owner_address)){
            let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
            vector::push_back(&mut myblog.articles, article_id);
        }else{
            let articles = vector::singleton(article_id);
            let myblog = MyBlog{
                name: string::utf8(b"MyBlog"),
                articles,
            };
            account_storage::global_move_to(ctx, owner, myblog);
        }
    }

    fun delete_article_from_myblog(ctx: &mut StorageContext, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        let (contains, index) = vector::index_of(&myblog.articles, &article_id);
        assert!(contains, error::not_found(ENOT_FOUND));
        vector::remove(&mut myblog.articles, index); 
    }

    /// Get owner's blog's articles
    public fun get_blog_articles(ctx: &StorageContext, owner_address: address): vector<ObjectID> {
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            vector::empty()
        }else{
            let myblog = account_storage::global_borrow<MyBlog>(ctx, owner_address);
            myblog.articles
        }
    }

    public entry fun create_article(
        ctx: &mut StorageContext,
        owner: signer,
        title: String,
        body: String,
    ) {
        let article_id = article::create_article(ctx, &owner, title, body);
        add_article_to_myblog(ctx, &owner, article_id);
    }

    public entry fun update_article(
        ctx: &mut StorageContext,
        owner: signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
    ) {
        article::update_article(ctx, &owner, id, new_title, new_body);
    }

    public entry fun delete_article(
        ctx: &mut StorageContext,
        owner: signer,
        id: ObjectID,
    ) {
        article::delete_article(ctx, &owner, id);
        delete_article_from_myblog(ctx, &owner, id);
    }
}
