// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::collection{
    use std::option::Option;
    use std::string::String;
    use moveos_std::object::ObjectID;
    use moveos_std::object_ref;
    use moveos_std::event;
    use moveos_std::context;
    use moveos_std::context::Context;
    use moveos_std::object;
    use moveos_std::object_ref::ObjectRef;
    use moveos_std::type_table;

    const EMutatorNotExist: u64 = 100;
    const ECollectionNotExist: u64 = 101;

    struct Collection has store{
        name: String,
        uri: String,
        creator: address,
        description: String,
        supply:  Supply,
        extend: type_table::TypeTable
    }

    struct Supply has store{
        current: u64,
        maximum: Option<u64>,
    }

    struct MutatorRef has store{
        collection: ObjectID,
    }

    // event
    struct CreateCollectionEvent{
        objectID: ObjectID,
        name: String,
        uri: String,
        creator: address,
        maximum: Option<u64>,
        description: String,
    }


    public fun create_collection(
        name: String,
        uri: String,
        creator: address,
        description: String,
        max_supply: Option<u64>,
        ctx: &mut Context
    ):ObjectRef<Collection> {

        let collection = Collection {
            name,
            uri,
            creator,
            description,
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

    public fun generate_mutator_ref(collection: ObjectRef<Collection>, ctx: &mut Context):ObjectRef<MutatorRef>{
        let mutator_ref = context::new_object_with_owner(
            ctx,
            object_ref::owner(&collection),
            MutatorRef {
                collection: object_ref::id(&collection),
            }
        );
        mutator_ref
    }

    // assert
    public fun assert_collection_exist_of_ref(collectionRef: &ObjectRef<Collection>){
        assert!( object_ref::exist_object(collectionRef), ECollectionNotExist);
    }

    public fun assert_collection_exist_of_id(collectionID: ObjectID, ctx: & Context){
        assert!( context::exist_object(ctx, collectionID), ECollectionNotExist);
        context::borrow_object<Collection>(ctx,collectionID);
    }

    public fun assert_mutator_exist_of_ref(mutatorRef: &ObjectRef<MutatorRef>){
        assert!( object_ref::exist_object(mutatorRef), EMutatorNotExist);
    }

    public fun assert_mutator_exist_of_id(mutatorID: ObjectID, ctx: & Context){
        assert!( context::exist_object(ctx, mutatorID), EMutatorNotExist);
        context::borrow_object<MutatorRef>(ctx, mutatorID);
    }

    public fun add_collection_extend<V: key>(mutator: &ObjectRef<MutatorRef>,val: V,ctx: &mut Context){
        assert_mutator_exist_of_ref(mutator);
        let collection = object_ref::borrow(mutator);
        assert_collection_exist_of_id(collection.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection>(ctx, collection.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        type_table::add( &mut collection_mut_ref.extend, val);
    }

    public fun borrow_collection_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):&V{
        assert_mutator_exist_of_ref(mutator);
        let collection = object_ref::borrow(mutator);
        assert_collection_exist_of_id(collection.collection, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collection.collection);
        let collection_ref = object::borrow(collection_object_ref);
        type_table::borrow(&collection_ref.extend)
    }

    public fun borrow_mut_collection_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):&mut V{
        assert_mutator_exist_of_ref(mutator);
        let collection = object_ref::borrow(mutator);
        assert_collection_exist_of_id(collection.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection>(ctx, collection.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        type_table::borrow_mut(&mut collection_mut_ref.extend)
    }

    public fun remove_collection_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context){
        assert_mutator_exist_of_ref(mutator);
        let collection = object_ref::borrow(mutator);
        assert_collection_exist_of_id(collection.collection, ctx);
        let collection_object_mut_ref = context::borrow_object_mut<Collection>(ctx, collection.collection);
        let collection_mut_ref = object::borrow_mut(collection_object_mut_ref);
        type_table::remove(&mut collection_mut_ref.extend)
    }

    public fun contains_collection_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context): bool{
        assert_mutator_exist_of_ref(mutator);
        let collection = object_ref::borrow(mutator);
        assert_collection_exist_of_id(collection.collection, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collection.collection);
        let collection_ref = object::borrow(collection_object_ref);
        type_table::contains<V>(&collection_ref.extend)
    }

    // view

    public fun get_collection_name(collectionID: ObjectID, ctx: &mut Context): String{
        assert_collection_exist_of_id(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.name
    }

    public fun get_collection_uri(collectionID: ObjectID, ctx: &mut Context): String{
        assert_collection_exist_of_id(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.uri
    }

    public fun get_collection_creator(collectionID: ObjectID, ctx: &mut Context): address{
        assert_collection_exist_of_id(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.creator
    }

    public fun get_collection_description(collectionID: ObjectID, ctx: &mut Context): String{
        assert_collection_exist_of_id(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.description
    }

    public fun get_collection_current_supply(collectionID: ObjectID, ctx: &mut Context): u64{
        assert_collection_exist_of_id(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.supply.current
    }

    public fun get_collection_maximum_supply(collectionID: ObjectID, ctx: &mut Context): Option<u64>{
        assert_collection_exist_of_id(collectionID, ctx);
        let collection_object_ref = context::borrow_object<Collection>(ctx, collectionID);
        let collection_ref = object::borrow(collection_object_ref);
        collection_ref.supply.maximum
    }

}
