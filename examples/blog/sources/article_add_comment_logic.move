// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::article_add_comment_logic {
    use moveos_std::object::Object;
    use rooch_examples::article::{Self, Article};
    use rooch_examples::comment;
    use rooch_examples::comment_added;
    use std::string::String;

    friend rooch_examples::article_aggregate;

    public(friend) fun verify(
        account: &signer,
        commenter: String,
        body: String,
        article_obj: &Object<Article>,
    ): article::CommentAdded {
        let _ = account;
        let comment_seq_id = article::current_comment_seq_id(article_obj) + 1;
        article::new_comment_added(
            article_obj,
            comment_seq_id,
            commenter,
            body,
            std::signer::address_of(account),
        )
    }

    public(friend) fun mutate(
        _account: &signer,
        comment_added: &article::CommentAdded,
        article_obj: &mut Object<Article>,
    ) {
        let comment_seq_id = article::next_comment_seq_id(article_obj);
        let commenter = comment_added::commenter(comment_added);
        let body = comment_added::body(comment_added);
        let owner = comment_added::owner(comment_added);
        let id = article::id(article_obj);
        let _ = id;
        let comment = comment::new_comment(
            comment_seq_id,
            commenter,
            body,
            owner,
        );
        article::add_comment(article_obj, comment);
    }

}
