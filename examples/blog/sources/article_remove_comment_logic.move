module rooch_examples::article_remove_comment_logic {
    use moveos_std::object::Object;
    use moveos_std::context::Context;
    use rooch_examples::article;
    use rooch_examples::comment;
    use rooch_examples::comment_removed;

    friend rooch_examples::article_aggregate;

    const ErrorNotOwnerAccount: u64 = 113;

    public(friend) fun verify(
        storage_ctx: &mut Context,
        account: &signer,
        comment_seq_id: u64,
        article_obj: &Object<article::Article>,
    ): article::CommentRemoved {
        let _ = storage_ctx;
        let comment = article::borrow_comment(article_obj, comment_seq_id);
        assert!(std::signer::address_of(account) == comment::owner(comment), ErrorNotOwnerAccount);
        article::new_comment_removed(
            article_obj,
            comment_seq_id,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut Context,
        _account: &signer,
        comment_removed: &article::CommentRemoved,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let comment_seq_id = comment_removed::comment_seq_id(comment_removed);
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        article::remove_comment(&mut article_obj, comment_seq_id);
        article_obj
    }

}
