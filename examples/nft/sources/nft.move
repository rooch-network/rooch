// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module nft::nft {
    use std::string::{Self, String};
    use nft::collection;
    use moveos_std::display;
    use moveos_std::object::{Self, ObjectID, UID, Object};
    use moveos_std::context::{Self, Context};
    #[test_only]
    use std::option;
    #[test_only]
    use rooch_framework::account;

    const ErrorCreatorNotMatch: u64 = 1;

    struct NFT has key,store {
        name: String,
        collection: ObjectID,
        creator: address,
    }

    fun init(ctx: &mut Context){
        let nft_display_object = display::object_display<NFT>(ctx);
        display::set_value(nft_display_object, string::utf8(b"name"), string::utf8(b"{ value.name }"));
        display::set_value(nft_display_object, string::utf8(b"owner"), string::utf8(b"{ owner }"));
        display::set_value(nft_display_object, string::utf8(b"creator"), string::utf8(b"{ value.creator }"));
        display::set_value(nft_display_object, string::utf8(b"uri"), string::utf8(b"https://base_url/{ collection }/{ id }"));
    }

    /// Mint a new NFT,
    public fun mint(
        collection_obj: &mut Object<collection::Collection>,
        nft_id: UID,
        name: String,
    ): Object<NFT> {
        let collection_id = object::id(collection_obj);
        let collection = object::borrow_mut(collection_obj);
        collection::increment_supply(collection);
        //NFT's creator should be the same as collection's creator?
        let creator = collection::creator(collection);
        let nft = NFT {
            name,
            collection: collection_id,
            creator,
        };
        
        let nft_obj = object::new(
            nft_id,
            nft
        );
        nft_obj
    }

    public fun burn (
        collection_obj: &mut Object<collection::Collection>, 
        nft_object: Object<NFT>,
    ) {
        let collection = object::borrow_mut(collection_obj);
        collection::decrement_supply(collection);
        let (
            NFT {
                name:_,
                collection:_,
                creator:_,
            }
        ) = object::remove<NFT>(nft_object);
    }

    // view

    public fun name(nft: &NFT): String {
        nft.name
    }


    public fun collection(nft: &NFT): ObjectID {
        nft.collection
    }

    public fun creator(nft: &NFT): address {
        nft.creator
    }

    /// Mint a new NFT and transfer it to sender
    /// The Collection is shared object, so anyone can mint a new NFT
    entry fun mint_entry(ctx: &mut Context, collection_obj: &mut Object<collection::Collection>, name: String) {
        let sender = context::sender(ctx);
        let nft_id = context::fresh_uid(ctx);
        let nft_obj = mint(collection_obj, nft_id, name);
        object::transfer(nft_obj, sender);
    }

     /// Update the base uri of the NFT
    /// the Collection is shared object, so we need to check the creator of collection, only the creator of collection can update the base uri
    entry fun update_base_uri(ctx: &mut Context, collection_obj: &Object<collection::Collection>, new_base_uri: String){
        let sender_address = context::sender(ctx);
        let collection = object::borrow(collection_obj);
        assert!(collection::creator(collection) == sender_address, ErrorCreatorNotMatch);
        let nft_display_obj = display::object_display<NFT>(ctx);
        string::append(&mut new_base_uri, string::utf8(b"{ collection }/{ id }"));
        display::set_value(nft_display_obj, string::utf8(b"uri"), new_base_uri);
    }

    #[test(sender = @nft)]
    public fun test_create_nft (sender: address){
        let ctx = context::new_test_context(sender);
        account::create_account_for_test(&mut ctx, sender);

        let collection_id = collection::create_collection(
            &mut ctx,
            string::utf8(b"test_collection_name1"),
            sender,
            string::utf8(b"test_collection_description1"),
            option::none(),
        );
        let nft_id = context::fresh_uid(&mut ctx);

        let collection_obj = context::borrow_mut_object_shared(&mut ctx, collection_id);
        let nft_obj = mint(
            collection_obj,
            nft_id,
            string::utf8(b"test_nft_1"),
        );
        object::transfer(nft_obj, sender);
        
        context::drop_test_context(ctx);
    }

}