module simple_blog::blog {
    use std::error;
    use std::signer;
    use std::string::{Self,String};
    use std::vector;
    use moveos_std::object_id::ObjectID;
    use moveos_std::context::Context;
    use moveos_std::account_storage;
    use simple_blog::article;

    const ErrorDataTooLong: u64 = 1;
    const ErrorNotFound: u64 = 2;

    struct MyBlog has key {
        name: String,
        articles: vector<ObjectID>,
    }

    /// This init function is called when the module is published
    /// The owner is the address of the account that publishes the module
    fun init(storage_ctx: &mut Context, owner: &signer) {
        // auto create blog for module publisher 
        create_blog(storage_ctx, owner);
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

    fun add_article_to_myblog(ctx: &mut Context, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            create_blog(ctx, owner);
        };
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        vector::push_back(&mut myblog.articles, article_id);
    }

    fun delete_article_from_myblog(ctx: &mut Context, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        let (contains, index) = vector::index_of(&myblog.articles, &article_id);
        assert!(contains, error::not_found(ErrorNotFound));
        vector::remove(&mut myblog.articles, index); 
    }

    /// Get owner's blog's articles
    public fun get_blog_articles(ctx: &Context, owner_address: address): vector<ObjectID> {
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            vector::empty()
        }else{
            let myblog = account_storage::global_borrow<MyBlog>(ctx, owner_address);
            myblog.articles
        }
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
        owner: signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
    ) {
        article::update_article(ctx, &owner, id, new_title, new_body);
    }

    public entry fun delete_article(
        ctx: &mut Context,
        owner: signer,
        id: ObjectID,
    ) {
        article::delete_article(ctx, &owner, id);
        delete_article_from_myblog(ctx, &owner, id);
    }
}
