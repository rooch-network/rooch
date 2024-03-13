// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::blog_delete_logic {
    
    use rooch_examples::blog;

    friend rooch_examples::blog_aggregate;

    public(friend) fun verify(
        
        account: &signer,
        blog: &blog::Blog,
    ): blog::BlogDeleted {
        
        let _ = account;
        blog::new_blog_deleted(
            blog,
        )
    }

    public(friend) fun mutate(
        
        _account: &signer,
        blog_deleted: &blog::BlogDeleted,
        blog: blog::Blog,
    ): blog::Blog {
        
        let _ = blog_deleted;
        blog
    }

}
