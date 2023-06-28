module rooch_examples::article_add_comment_logic {
    use std::string::String;

    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;
    use rooch_examples::comment;
    use rooch_examples::comment_added;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        comment_seq_id: u64,
        commenter: String,
        body: String,
        article_obj: &Object<article::Article>,
    ): article::CommentAdded {
        let _ = storage_ctx;
        let _ = account;
        article::new_comment_added(
            article_obj,
            comment_seq_id,
            commenter,
            body,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut StorageContext,
        comment_added: &article::CommentAdded,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let comment_seq_id = comment_added::comment_seq_id(comment_added);
        let commenter = comment_added::commenter(comment_added);
        let body = comment_added::body(comment_added);
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        let comment = comment::new_comment(
            comment_seq_id,
            commenter,
            body,
        );
        article::add_comment(storage_ctx, &mut article_obj, comment);
        article_obj
    }
}
