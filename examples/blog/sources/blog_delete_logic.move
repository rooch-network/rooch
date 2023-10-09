module rooch_examples::blog_delete_logic {
    use moveos_std::context::Context;
    use rooch_examples::blog;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        ctx: &mut Context,
        account: &signer,
        blog: &blog::Blog,
    ): blog::BlogDeleted {
        let _ = ctx;
        let _ = account;
        blog::new_blog_deleted(
            blog,
        )
    }

    public(friend) fun mutate(
        ctx: &mut Context,
        _account: &signer,
        blog_deleted: &blog::BlogDeleted,
        blog: blog::Blog,
    ): blog::Blog {
        let _ = ctx;
        let _ = blog_deleted;
        blog
    }

}
