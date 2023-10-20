// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module nft::collection{
    use std::option;
    use std::option::Option;
    use std::string::String;
    use rooch_framework::display::{Self, Display};
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use moveos_std::object_ref::{Self, ObjectRef};
    use moveos_std::type_table;

    friend nft::nft;

    const ErrorMutatorNotExist: u64 = 1;
    const ErrorCollectionNotExist: u64 = 2;
    const ErrorCollectionMaximumSupply: u64 = 3;

    struct Collection<phantom T> has key{
        name: String,
        uri: String,
        creator: address,
        supply:  Supply,
        extend: type_table::TypeTable
    }

    struct Supply has store{
        current: u64,
        maximum: Option<u64>,
    }

    struct MutatorRef has key,store{
        collection: ObjectID,
    }

    struct CreateCollectionEvent{
        objectID: ObjectID,
        name: String,
        uri: String,
        creator: address,
        maximum: Option<u64>,
        description: String,
    }

    public(friend) fun create_collection<T>(
        name: String,
        uri: String,
        creator: address,
        description: String,
        max_supply: Option<u64>,
        ctx: &mut Context
    ):ObjectRef<Collection<T>> {

        let collection = Collection {
            name,
            uri,
            creator,
            supply: Supply {
                current: 0,
                maximum: max_supply,
            },
            extend: type_table::new(ctx)
        };

        let object_ref = context::new_object_with_owner(
            ctx,
            creator,
            collection
        );

        event::emit(
            ctx,
            CreateCollectionEvent {
                objectID: object_ref::id(&object_ref),
                name,
                uri,
                creator,
                maximum: max_supply,
                description,
            }
        );
        object_ref
    }

    public fun generate_mutator_ref<T>(collection: &ObjectRef<Collection<T>>):MutatorRef{
        MutatorRef {
            collection: object_ref::id(collection),
        }
    }

    public(friend) fun new_display<T>(ctx: &mut Context):ObjectRef<Display<Collection<T>>>{
        display::new<Collection<T>>(ctx)
    }

    public fun destroy_mutator_ref(mutator_ref :MutatorRef):ObjectID{
        let MutatorRef {
            collection,
        } = mutator_ref;
        collection
    }

    public fun get_collection_id(mutator: &MutatorRef): ObjectID{
        mutator.collection
    }


    public(friend) fun increment_supply<T>(mutator: &MutatorRef, ctx: &mut Context): Option<u64>{
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection<T>>(ctx, mutator.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        collection_mut_ref.supply.current = collection_mut_ref.supply.current + 1;
        if(option::is_some(&collection_mut_ref.supply.maximum)){
            assert!(collection_mut_ref.supply.current <= *option::borrow(&collection_mut_ref.supply.maximum), ErrorCollectionMaximumSupply);
            option::some(collection_mut_ref.supply.current)
        }else{
            option::none<u64>()
        }
    }

    public (friend) fun decrement_supply<T>(mutator: &MutatorRef, ctx: &mut Context): Option<u64>{
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection<T>>(ctx, mutator.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        collection_mut_ref.supply.current = collection_mut_ref.supply.current - 1;
        if(option::is_some(&collection_mut_ref.supply.maximum)){
            option::some(collection_mut_ref.supply.current)
        }else{
            option::none<u64>()
        }
    }

    // assert
    public fun assert_collection_exist_of_ref<T>(collectionRef: &ObjectRef<Collection<T>>){
        assert!( object_ref::exist_object(collectionRef), ErrorCollectionNotExist);
    }

    public fun assert_collection_exist_of_id<T>(collectionID: ObjectID, ctx: & Context){
        assert!( context::exist_object(ctx, collectionID), ErrorCollectionNotExist);
        context::borrow_object<Collection<T>>(ctx,collectionID);
    }

    #[private_generics(V)]
    public fun add_extend<T,V: key>(mutator: &MutatorRef, val: V, ctx: &mut Context){
        add_extend_internal<T,V>(mutator, val, ctx);
    }

    #[private_generics(V)]
    public fun borrow_extend<T,V: key>(mutator: &MutatorRef, ctx: &mut Context):&V{
        borrow_extend_internal<T,V>(mutator, ctx)
    }

    #[private_generics(V)]
    public fun borrow_mut_extend<T,V: key>(mutator: &MutatorRef, ctx: &mut Context):&mut V{
        borrow_mut_extend_internal<T,V>(mutator, ctx)
    }

    #[private_generics(V)]
    public fun remove_extend<T,V: key>(mutator: &MutatorRef, ctx: &mut Context):V{
        remove_extend_internal<T,V>(mutator, ctx)
    }

    public fun contains_extend<T,V: key>(mutator: &MutatorRef, ctx: &mut Context): bool{
        contains_extend_internal<T,V>(mutator, ctx)
    }


    fun add_extend_internal<T,V: key>(mutator: &MutatorRef,val: V,ctx: &mut Context){
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection<T>>(ctx, mutator.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        type_table::add( &mut collection_mut_ref.extend, val);
    }

    fun borrow_extend_internal<T, V: key>(mutator: &MutatorRef, ctx: &mut Context):&V{
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, mutator.collection);
        let collection_ref = object::borrow(collection_object_ref);
        type_table::borrow(&collection_ref.extend)
    }

    fun borrow_mut_extend_internal<T,V: key>(mutator: &MutatorRef, ctx: &mut Context):&mut V{
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection<T>>(ctx, mutator.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        type_table::borrow_mut(&mut collection_mut_ref.extend)
    }

    fun remove_extend_internal<T,V: key>(mutator: &MutatorRef, ctx: &mut Context):V{
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection<T>>(ctx, mutator.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        type_table::remove<V>(&mut collection_mut_ref.extend)
    }

    fun contains_extend_internal<T,V: key>(mutator: &MutatorRef, ctx: &mut Context): bool{
        assert_collection_exist_of_id<T>(mutator.collection, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, mutator.collection);
        let collection_ref = object::borrow(collection_object_ref);
        type_table::contains<V>(&collection_ref.extend)
    }

    // view
    public fun get_collection_name<T>(collectionID: ObjectID, ctx: &mut Context): String{
        assert_collection_exist_of_id<T>(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.name
    }

    public fun get_collection_uri<T>(collectionID: ObjectID, ctx: &mut Context): String{
        assert_collection_exist_of_id<T>(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.uri
    }

    public fun get_collection_creator<T>(collectionID: ObjectID, ctx: &mut Context): address{
        assert_collection_exist_of_id<T>(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.creator
    }

    public fun get_collection_current_supply<T>(collectionID: ObjectID, ctx: &mut Context): u64{
        assert_collection_exist_of_id<T>(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.supply.current
    }

    public fun get_collection_maximum_supply<T>(collectionID: ObjectID, ctx: &mut Context): Option<u64>{
        assert_collection_exist_of_id<T>(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection<T>>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.supply.maximum
    }

}
