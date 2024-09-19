// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::bitseed {

    use std::option::{Option};
    use std::string::{Self, String};
    
    use moveos_std::object::{Self, Object};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::copyable_any::Any;

    use bitcoin_move::ord::{Self, InscriptionID};

    const METAPROTOCOL : vector<u8> = b"bitseed";
    const BIT_SEED_DEPLOY: vector<u8> = b"bitseed_deploy";
    const BIT_SEED_MINT: vector<u8> = b"bitseed_mint";
    const BIT_SEED_GENERATOR_TICK: vector<u8> = b"generator";

    const ErrorBitseedNotMergeable: u64 = 1;
    const ErrorBitseedNotSplittable: u64 = 2;
    const ErrorInvalidAmount: u64 = 3;

    friend rooch_nursery::tick_info;
    friend rooch_nursery::genesis;
    friend rooch_nursery::inscribe_factory;

    /// Bitseed is a SFT asset type.
    struct Bitseed has key,store{
        /// The metaprotocol of the bitseed.
        /// It is the namespace of the tick.
        metaprotocol: String,
        /// The tick of the bitseed.
        tick: String,
        /// A unique identifier for the bitseed.
        /// Bitseed leap between L1 and L2
        /// the container is changed, but the bid is still the same.
        bid: address,
        /// The amount of the bitseed.
        amount: u64,
        /// Indicate the body type of the bitseed.
        content_type: Option<String>,
        /// The body of the bitseed.
        body: vector<u8>,
        /// The attributes of the bitseed.
        attributes: SimpleMap<String, Any>,
        /// The content and content attributes hash of the bitseed.
        /// This is used to check if two bitseeds are repeated.
        content_attributes_hash: address,
    }

    public fun default_metaprotocol(): String{
        string::utf8(METAPROTOCOL)
    }

    // ================== Bitseed ================== //


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
            attributes: simple_map::new(),
            //TODO calculate the hash
            content_attributes_hash: @0x0,
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
        bitseed1.content_attributes_hash == bitseed2.content_attributes_hash
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

    public(friend) fun burn(bitseed_obj: Object<Bitseed>) : u64{
        let bitseed = object::remove(bitseed_obj);
        let amount = bitseed.amount;
        drop(bitseed);
        amount
    }

    // ==================== Get functions ==================== //

    public fun metaprotocol(bitseed: &Bitseed): String {
        bitseed.metaprotocol
    }
    
    public fun amount(bitseed: &Bitseed): u64 {
        bitseed.amount
    }

    public fun tick(bitseed: &Bitseed): String {
        bitseed.tick
    }

    public fun bid(bitseed: &Bitseed): address {
        bitseed.bid
    }

    public fun content_type(bitseed: &Bitseed): &Option<String> {
        &bitseed.content_type
    }

    public fun body(bitseed: &Bitseed): &vector<u8> {
        &bitseed.body
    }

    public fun attributes(bitseed: &Bitseed): &SimpleMap<String, Any> {
        &bitseed.attributes
    }

    public fun content_attributes_hash(bitseed: &Bitseed): address {
        bitseed.content_attributes_hash
    }

    // ========= Metaprotocol delegate function ========= //

    public(friend) fun seal_metaprotocol_validity(inscription_id: InscriptionID, is_valid: bool, invalid_reason: Option<String>){
        ord::seal_metaprotocol_validity<Bitseed>(inscription_id, is_valid, invalid_reason)
    }

    public(friend) fun add_metaprotocol_attachment(inscription_id: InscriptionID, attachment: Object<Bitseed>){
        ord::add_metaprotocol_attachment<Bitseed>(inscription_id, attachment)
    }

    public(friend) fun exists_metaprotocol_attachment(inscription_id: InscriptionID): bool{
        ord::exists_metaprotocol_attachment<Bitseed>(inscription_id)
    }

    public(friend) fun remove_metaprotocol_attachment(inscription_id: InscriptionID): Object<Bitseed>{
        ord::remove_metaprotocol_attachment<Bitseed>(inscription_id)
    }

    fun drop(bitseed: Bitseed) {
        let Bitseed {
            metaprotocol:_,
            tick:_,
            bid:_,
            amount:_,
            content_type:_,
            body:_,
            attributes:_,
            content_attributes_hash:_,
        } = bitseed;
    }

}