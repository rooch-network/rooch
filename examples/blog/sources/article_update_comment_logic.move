module rooch_examples::article_update_comment_logic {
    use moveos_std::object::Object;
    use moveos_std::context::Context;
    use rooch_examples::article;
    use rooch_examples::comment;
    use rooch_examples::comment_updated;
    use std::string::String;

    friend rooch_examples::article_aggregate;

    const ErrorNotOwnerAccount: u64 = 113;

    public(friend) fun verify(
        storage_ctx: &mut Context,
        account: &signer,
        comment_seq_id: u64,
        commenter: String,
        body: String,
        owner: address,
        article_obj: &Object<article::Article>,
    ): article::CommentUpdated {
        let _ = storage_ctx;
        let comment = article::borrow_comment(article_obj, comment_seq_id);
        assert!(std::signer::address_of(account) == comment::owner(comment), ErrorNotOwnerAccount);
        article::new_comment_updated(
            article_obj,
            comment_seq_id,
            commenter,
            body,
            owner,
        )
    }

    public(friend) fun mutate(
        storage_ctx: &mut Context,
        _account: &signer,
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
