// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module nft::nft {
    use std::string::{Self, String};
    use nft::collection;
    use rooch_framework::display;
    use moveos_std::object_ref::{Self, ObjectRef};
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{ObjectID};
    #[test_only]
    use std::option;
    #[test_only]
    use rooch_framework::account;

    const ErrorNftNotExist: u64 = 1;
    const ErrorMutatorNotExist: u64 = 2;
    const ErrorBurnerNotExist: u64 = 3;

    struct NFT has key,store {
        name: String,
        uri: String,
        collection: ObjectID,
        creator: address,
    }

    fun init(ctx: &mut Context){
        let nft_display_object = display::new<NFT>(ctx);
        display::set(&mut nft_display_object, string::utf8(b"name"), string::utf8(b"{ name }"));
        display::set(&mut nft_display_object, string::utf8(b"uri"), string::utf8(b"{ uri }"));
        object_ref::to_permanent(nft_display_object);
    }

    /// Mint a new NFT,
    public fun mint(
        ctx: &mut Context,
        collection_obj: &mut ObjectRef<collection::Collection>,
        name: String,
        uri: String,
    ): ObjectRef<NFT> {
        let collection_id = object_ref::id(collection_obj);
        let collection = object_ref::borrow_mut(collection_obj);
        collection::increment_supply(collection);
        //NFT's creator should be the same as collection's creator?
        let creator = collection::creator(collection);
        let nft = NFT {
            name,
            uri,
            collection: collection_id,
            creator,
        };
        
        let nft_obj = context::new_object(
            ctx,
            nft
        );
        nft_obj
    }

    public fun burn (
        collection_obj: &mut ObjectRef<collection::Collection>, 
        nft_object: ObjectRef<NFT>,
    ) {
        let collection = object_ref::borrow_mut(collection_obj);
        collection::decrement_supply(collection);
        let (
            NFT {
                name:_,
                uri:_,
                collection:_,
                creator:_,
            }
        ) = object_ref::remove<NFT>(nft_object);
    }

    // view

    public fun name(nft: &NFT): String {
        nft.name
    }

    public fun uri(nft: &NFT): String {
        nft.uri
    }

    public fun collection(nft: &NFT): ObjectID {
        nft.collection
    }

    public fun creator(nft: &NFT): address {
        nft.creator
    }

    /// Mint a new NFT and transfer it to sender
    /// Because only the creator of the collection can get `&mut ObjectRef<collection::Collection>`
    /// So, only the creator of the collection can mint a new NFT
    /// If we want to allow other people to mint NFT, we need to make the `ObjectRef<collection::Collection>` to shared
    entry fun mint_entry(ctx: &mut Context, collection_obj: &mut ObjectRef<collection::Collection>, name: String, uri: String) {
        let sender = context::sender(ctx);
        let nft_obj = mint(ctx, collection_obj, name, uri);
        object_ref::transfer(&mut nft_obj, sender);
        //Because the NFT becomes permanent Object here, we can not to burn it.
        //Maybe we need to design a NFTGallery to store all the NFTs of user.
        object_ref::to_permanent(nft_obj);
    }

    #[test(sender = @nft)]
    public fun test_create_nft (sender: address){
        let storage_context = context::new_test_context(sender);
        let ctx = &mut storage_context;
        account::create_account_for_test(ctx, sender);

        let collection_obj = collection::create_collection(
            ctx,
            string::utf8(b"test_collection_name1"),
            string::utf8(b"test_collection_uri1"),
            sender,
            string::utf8(b"test_collection_description1"),
            option::none(),
        );

        
        let nft_obj = mint(
            ctx,
            &mut collection_obj,
            string::utf8(b"test_nft_1"),
            string::utf8(b"test_nft_uri"),
        );
        object_ref::transfer(&mut nft_obj, sender);

        burn(&mut collection_obj, nft_obj);

        object_ref::to_permanent(collection_obj);

        context::drop_test_context(storage_context);
    }

}