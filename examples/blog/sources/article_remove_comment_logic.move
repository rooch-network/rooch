// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::article_remove_comment_logic {
    use moveos_std::object_ref::ObjectRef;
    use rooch_examples::article::{Self, Article};
    use rooch_examples::comment;
    use rooch_examples::comment_removed;

    friend rooch_examples::article_aggregate;

    const ErrorNotOwnerAccount: u64 = 113;

    public(friend) fun verify(
        account: &signer,
        comment_seq_id: u64,
        article_obj: &ObjectRef<Article>,
    ): article::CommentRemoved {
        let comment = article::borrow_comment(article_obj, comment_seq_id);
        assert!(std::signer::address_of(account) == comment::owner(comment), ErrorNotOwnerAccount);
        article::new_comment_removed(
            article_obj,
            comment_seq_id,
        )
    }

    public(friend) fun mutate(
        _account: &signer,
        comment_removed: &article::CommentRemoved,
        article_obj: &mut ObjectRef<Article>,
    ) {
        let comment_seq_id = comment_removed::comment_seq_id(comment_removed);
        let id = article::id(article_obj);
        let _ = id;
        article::remove_comment(article_obj, comment_seq_id);
    }

}
