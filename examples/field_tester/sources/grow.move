// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module field_tester::grow {
    use moveos_std::object::{Self};
    use moveos_std::table::{Self, Table};

    struct Grow has key, store {
        vote_detail: Table<address, u256>,
    }

    fun init(){
        let grow_obj_id = object::named_object_id<Grow>();
        std::debug::print(&grow_obj_id);
        let grow_obj= object::new_named_object(Grow{
            vote_detail: table::new(),
        });
        object::to_shared(grow_obj);
        // object::transfer(grow_obj, moveos_std::tx_context::sender())
    }

    public entry fun vote_entry(
        _account: &signer,
        vote_addr: address,
        vote_val: u256,
    ){
        vote(_account, vote_addr,  vote_val)
    }

    public fun vote(
        _account: &signer,
        vote_addr: address,
        vote_val: u256,
    ) {
        let grow_obj_id = object::named_object_id<Grow>();
        let grow_obj = object::borrow_mut_object_shared<Grow>(grow_obj_id);
        let grow = object::borrow_mut(grow_obj);
        let vote_detail = table::borrow_mut_with_default(&mut grow.vote_detail, vote_addr, 0);
        *vote_detail = *vote_detail + vote_val;
    }
}