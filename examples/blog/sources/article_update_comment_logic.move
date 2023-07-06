module rooch_examples::article_update_comment_logic {
    use moveos_std::object::Object;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::article;
    use rooch_examples::comment;
    use rooch_examples::comment_updated;
    use std::string::String;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        comment_seq_id: u64,
        commenter: String,
        body: String,
        owner: address,
        article_obj: &Object<article::Article>,
    ): article::CommentUpdated {
        let _ = storage_ctx;
        let _ = account;
        let comment = article::borrow_comment(article_obj, comment_seq_id);
        let _ = comment;
        assert!(std::signer::address_of(account) == comment::owner(comment), 111);
        article::new_comment_updated(
            article_obj,
            comment_seq_id,
            commenter,
            body,
            owner,
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
        let owner = comment_updated::owner(comment_updated);
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        let comment = article::borrow_mut_comment(&mut article_obj, comment_seq_id);
        comment::set_commenter(comment, commenter);
        comment::set_body(comment, body);
        comment::set_owner(comment, owner);
        article_obj
    }

}
