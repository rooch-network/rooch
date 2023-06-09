module rooch_demo::article_remove_comment_logic {
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_demo::article;
    use rooch_demo::comment_removed;

    friend rooch_demo::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        comment_seq_id: u64,
        article_obj: &Object<article::Article>,
    ): article::CommentRemoved {
        let _ = storage_ctx;
        let _ = account;
        article::new_comment_removed(
            article_obj,
            comment_seq_id,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
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
