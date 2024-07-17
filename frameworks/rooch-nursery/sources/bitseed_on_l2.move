// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Bitseed on L2, in the future, this module will merge into the bitseed module.
module rooch_nursery::bitseed_on_l2 {

    use std::option::{Option};
    use std::string::String;
    use moveos_std::object::{Self, Object};

    const ErrorBitseedNotMergeable: u64 = 1;
    const ErrorBitseedNotSplittable: u64 = 2;
    const ErrorInvalidAmount: u64 = 3;

    friend rooch_nursery::tick_info;

    /// Bitseed is a SFT asset type.
    struct Bitseed has key,store{
        /// The metaprotocol of the bitseed.
        /// It is the namespace of the tick.
        metaprotocol: String,
        /// The tick of the bitseed.
        tick: String,
        /// A unique identifier for the bitseed.
        /// Bitseed leap between L1 and L2, the container is changed, but the bid is still the same.
        bid: address,
        /// The amount of the bitseed.
        amount: u64,
        /// Indicate the body type of the bitseed.
        content_type: Option<String>,
        /// The body of the bitseed.
        body: vector<u8>,
    }

    public(friend) fun new(
        metaprotocol: String,
        tick: String,
        bid: address,
        amount: u64,
        content_type: Option<String>,
        body: vector<u8>,
    ) : Object<Bitseed> {
        let bitseed = Bitseed {
            metaprotocol,
            tick,
            bid,
            amount,
            content_type,
            body,
        };
        object::new(bitseed)
    }

    /// Check if the two bitseeds are the same type.
    public fun is_same_type(bitseed1_obj: &Object<Bitseed>, bitseed2_obj: &Object<Bitseed>): bool {
        let bitseed1 = object::borrow(bitseed1_obj);
        let bitseed2 = object::borrow(bitseed2_obj);
        bitseed1.metaprotocol == bitseed2.metaprotocol 
        && bitseed1.tick == bitseed2.tick
    }

    /// Check if the two bitseeds are mergeable.
    public fun is_mergeable(bitseed1_obj: &Object<Bitseed>, bitseed2_obj: &Object<Bitseed>): bool {
        if(!is_same_type(bitseed1_obj, bitseed2_obj)){
            return false
        };
        let bitseed1 = object::borrow(bitseed1_obj);
        let bitseed2 = object::borrow(bitseed2_obj);
        bitseed1.content_type == bitseed2.content_type
        && bitseed1.body == bitseed2.body
    }

    public fun merge(bitseed1_obj: &mut Object<Bitseed>, bitseed2_obj: Object<Bitseed>) {
        assert!(is_mergeable(bitseed1_obj, &bitseed2_obj), ErrorBitseedNotMergeable);
        let bitseed1 = object::borrow_mut(bitseed1_obj);
        let bitseed2 = object::remove(bitseed2_obj);
        bitseed1.amount = bitseed1.amount + bitseed2.amount;
        drop(bitseed2);
    }
    
    /// Check if the bitseed is splittable.
    public fun is_splitable(bitseed_obj: &Object<Bitseed>): bool {
        object::borrow(bitseed_obj).amount > 1 && object::field_size(bitseed_obj) == 0
    }

    /// Split the bitseed and return the new bitseed.
    public fun split(
        bitseed_obj: &mut Object<Bitseed>,
        amount: u64,
    ) : Object<Bitseed> {
        let field_size = object::field_size(bitseed_obj);
        let bitseed = object::borrow_mut(bitseed_obj);
        assert!(amount > 0 && amount < bitseed.amount, ErrorInvalidAmount);
        assert!(field_size == 0, ErrorBitseedNotSplittable);

        let original_amount = bitseed.amount;
        bitseed.amount = original_amount - amount;

        let split_bitseed = new(
            bitseed.metaprotocol,
            bitseed.tick,
            bitseed.bid,
            amount,
            bitseed.content_type,
            bitseed.body,
        );
        split_bitseed
    }

    fun drop(bitseed: Bitseed) {
        let Bitseed {
            metaprotocol:_,
            tick:_,
            bid:_,
            amount:_,
            content_type:_,
            body:_,
        } = bitseed;
    }
}