module rooch_demo::article_update_comment_logic {
    use std::string::String;

    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_demo::article;
    use rooch_demo::comment;
    use rooch_demo::comment_updated;

    friend rooch_demo::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        comment_seq_id: u64,
        commenter: String,
        body: String,
        article_obj: &Object<article::Article>,
    ): article::CommentUpdated {
        let _ = storage_ctx;
        let _ = account;
        article::new_comment_updated(
            article_obj,
            comment_seq_id,
            commenter,
            body,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        comment_updated: &article::CommentUpdated,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let comment_seq_id = comment_updated::comment_seq_id(comment_updated);
        let commenter = comment_updated::commenter(comment_updated);
        let body = comment_updated::body(comment_updated);
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        let comment = article::borrow_mut_comment(&mut article_obj, comment_seq_id);
        comment::set_commenter(comment, commenter);
        comment::set_body(comment, body);
        article_obj
    }
}
