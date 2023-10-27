// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module nft::collection{
    use std::option;
    use std::option::Option;
    use std::string::{Self, String};
    use rooch_framework::display;
    use moveos_std::object::{ObjectID};
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use moveos_std::object_ref::{Self, ObjectRef};

    friend nft::nft;

    const ErrorMutatorNotExist: u64 = 1;
    const ErrorCollectionNotExist: u64 = 2;
    const ErrorCollectionMaximumSupply: u64 = 3;

    struct Collection has key{
        name: String,
        uri: String,
        creator: address,
        supply:  Supply,
    }

    struct Supply has store{
        current: u64,
        maximum: Option<u64>,
    }

    struct CreateCollectionEvent{
        objectID: ObjectID,
        name: String,
        uri: String,
        creator: address,
        maximum: Option<u64>,
        description: String,
    }

    fun init(ctx: &mut Context){
        let collection_display_obj = display::new<Collection>(ctx); 
        display::set(&mut collection_display_obj, string::utf8(b"name"), string::utf8(b"{ name }"));
        display::set(&mut collection_display_obj, string::utf8(b"uri"), string::utf8(b"{ uri }"));
        display::set(&mut collection_display_obj, string::utf8(b"description"), string::utf8(b"{ description }"));
        display::set(&mut collection_display_obj, string::utf8(b"creator"), string::utf8(b"{ creator }"));
        display::set(&mut collection_display_obj, string::utf8(b"supply"), string::utf8(b"{ supply }"));
        object_ref::to_permanent(collection_display_obj);
    }

    /// Create a new collection Object
    public fun create_collection(
        ctx: &mut Context,
        name: String,
        uri: String,
        creator: address,
        description: String,
        max_supply: Option<u64>,
    ) : ObjectRef<Collection> {

        let collection = Collection {
            name,
            uri,
            creator,
            supply: Supply {
                current: 0,
                maximum: max_supply,
            },
        };

        let collection_obj = context::new_object(
            ctx,
            collection
        );
        event::emit(
            ctx,
            CreateCollectionEvent {
                objectID: object_ref::id(&collection_obj),
                name,
                uri,
                creator,
                maximum: max_supply,
                description,
            }
        );
        object_ref::transfer_extend(&mut collection_obj, creator);
        collection_obj
    }

    public(friend) fun increment_supply(collection: &mut Collection): Option<u64>{
        collection.supply.current = collection.supply.current + 1;
        if(option::is_some(&collection.supply.maximum)){
            assert!(collection.supply.current <= *option::borrow(&collection.supply.maximum), ErrorCollectionMaximumSupply);
            option::some(collection.supply.current)
        }else{
            option::none<u64>()
        }
    }

    public(friend) fun decrement_supply(collection: &mut Collection): Option<u64>{
        collection.supply.current = collection.supply.current - 1;
        if(option::is_some(&collection.supply.maximum)){
            option::some(collection.supply.current)
        }else{
            option::none<u64>()
        }
    }

    // view
    public fun name(collection: &Collection): String{
        collection.name
    }

    public fun uri(collection: &Collection): String{
        collection.uri
    }

    public fun creator(collection: &Collection): address{
        collection.creator
    }

    public fun current_supply(collection: &Collection): u64{
        collection.supply.current
    }

    public fun maximum_supply(collection: &Collection): Option<u64>{
        collection.supply.maximum
    }

}
