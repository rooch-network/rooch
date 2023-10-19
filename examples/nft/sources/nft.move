// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module creator::nft {
    use std::option::{Option};
    use std::string::String;
    use creator::collection::Collection;
    use rooch_framework::display;
    use rooch_framework::display::Display;
    use moveos_std::object_ref;
    use moveos_std::object;
    use creator::collection;
    use moveos_std::object_ref::ObjectRef;
    use moveos_std::context::Context;
    use moveos_std::context;
    use moveos_std::type_table;
    use moveos_std::object::ObjectID;
    use moveos_std::type_table::TypeTable;
    #[test_only]
    use std::option;
    #[test_only]
    use std::string;
    #[test_only]
    use rooch_framework::account;

    const ErrorNftNotExist: u64 = 1;
    const ErrorMutatorNotExist: u64 = 2;
    const ErrorBurnerNotExist: u64 = 3;

    struct NFT<phantom T> has key,store {
        name: String,
        uri: String,
        collection: ObjectID,
        creator: address,
        extend: TypeTable
    }

    struct MutatorRef<phantom T> has key,store {
        nft: ObjectID,
    }

    struct BurnerRef<phantom T> has key,store {
        nft: ObjectID,
    }

    #[private_generics(T)]
    public fun create_collection<T>(
        name: String,
        uri: String,
        creator: address,
        description: String,
        supply: Option<u64>,
        ctx: &mut Context
    ):(ObjectRef<Collection<T>>,ObjectRef<Display<Collection<T>>>,ObjectRef<Display<NFT<T>>>) {
        let collection_object_ref = collection::create_collection<T>(
            name,
            uri,
            creator,
            description,
            supply,
            ctx
        );
        let collection_display_object_ref =  collection::new_display<T>(ctx);
        let nft_display_object_ref =  display::new<NFT<T>>(ctx);

        (collection_object_ref, collection_display_object_ref, nft_display_object_ref)
    }

    #[private_generics(T)]
    public fun mint<T>(
        name: String,
        uri: String,
        mutator_ref: &collection::MutatorRef,
        creator: address,
        ctx: &mut Context
    ): ObjectRef<NFT<T>> {
        let nft = NFT<T> {
            name,
            uri,
            collection: collection::get_collection_id(mutator_ref),
            creator,
            extend: type_table::new(ctx)
        };

        collection::increment_supply<T>(mutator_ref, ctx);

        let object_ref = context::new_object_with_owner(
            ctx,
            creator,
            nft
        );

        object_ref
    }

    public fun burn<T> (
        burn_ref: &BurnerRef<T>,
        mutator_ref: & collection::MutatorRef,
        ctx: &mut Context
    ) {
        assert_nft_exist_of_id<T>(burn_ref.nft, ctx);
        collection::decrement_supply<T>(mutator_ref, ctx);
        let (
            _,
            _,
            NFT {
                name:_,
                uri:_,
                collection:_,
                creator:_,
                extend
            }
        ) = context::remove_object<NFT<T>>(ctx, burn_ref.nft);
        if(type_table::contains<Display<T>>( &extend )){
           type_table::remove<Display<T>>( &mut extend);
        };
        type_table::destroy_empty(extend)
    }

    public fun generate_mutator_ref<T>(nft_object_ref: &ObjectRef<NFT<T>>):MutatorRef<T>{
        MutatorRef {
            nft: object_ref::id(nft_object_ref),
        }
    }

    public fun destroy_mutator_ref<T>(mutator_ref :MutatorRef<T>):ObjectID{
        let MutatorRef {
            nft
        } =  mutator_ref ;
        nft
    }

    public fun generate_burner_ref<T>(nft_object_ref: &ObjectRef<NFT<T>>):BurnerRef<T>{
        BurnerRef<T> {
            nft: object_ref::id(nft_object_ref),
        }
    }

    public fun destroy_burner_ref<T>(burner_ref :BurnerRef<T>):ObjectID{
        let BurnerRef {
            nft
        } = burner_ref;
        nft
    }

    // assert
    public fun assert_nft_exist_of_id<T>(objectId: ObjectID, ctx: &Context) {
        assert!(context::exist_object(ctx, objectId), ErrorNftNotExist);
        context::borrow_object<NFT<T>>(ctx, objectId);
    }

    public fun assert_nft_exist_of_ref<T>(nft_object_ref: &ObjectRef<NFT<T>>) {
        assert!(object_ref::exist_object(nft_object_ref), ErrorNftNotExist);
    }

    #[private_generics(V)]
    public fun add_extend<T,V: key>(mutator: &MutatorRef<T>, val: V, ctx: &mut Context){
        add_extend_internal<T,V>(mutator, val, ctx);
    }

    public fun borrow_extend<T,V: key>(mutator: &MutatorRef<T>, ctx: &mut Context):&V{
        borrow_extend_internal<T,V>(mutator, ctx)
    }

    #[private_generics(V)]
    public fun borrow_mut_extend<T,V: key>(mutator: &MutatorRef<T>, ctx: &mut Context):&mut V{
        borrow_mut_extend_internal<T,V>(mutator, ctx)
    }

    #[private_generics(V)]
    public fun remove_extend<T,V: key>(mutator: &MutatorRef<T>, ctx: &mut Context):V{
        remove_extend_internal<T,V>(mutator, ctx)
    }

    public fun contains_extend<T,V: key>(mutator: &MutatorRef<T>, ctx: &mut Context): bool{
        contains_extend_internal<T,V>(mutator, ctx)
    }

    fun add_extend_internal<T,V: key>(mutator: &MutatorRef<T>,val: V,ctx: &mut Context) {
        let nft_object_mut_ref = context::borrow_object_mut<NFT<T>>(ctx, mutator.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::add( &mut nft_mut_ref.extend, val);
    }

    fun borrow_extend_internal<T,V: key>(mutator: &MutatorRef<T>,ctx: &Context): &V {
        let nft_object_ref = context::borrow_object<NFT<T>>(ctx, mutator.nft);
        let nft_mut_ref = object::borrow(nft_object_ref);
        type_table::borrow(&nft_mut_ref.extend)
    }

    fun borrow_mut_extend_internal<T,V: key>(mutator: &MutatorRef<T>,ctx: &mut Context): &mut V {
        let nft_object_mut_ref = context::borrow_object_mut<NFT<T>>(ctx, mutator.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::borrow_mut(&mut nft_mut_ref.extend)
    }

    fun remove_extend_internal<T,V: key>(mutator: &MutatorRef<T>,ctx: &mut Context):V {
        let nft_object_mut_ref = context::borrow_object_mut<NFT<T>>(ctx, mutator.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::remove<V>(&mut nft_mut_ref.extend)
    }

    fun contains_extend_internal<T,V: key>(mutator: &MutatorRef<T>,ctx: &Context): bool {
        let nft_object_ref = context::borrow_object<NFT<T>>(ctx, mutator.nft);
        let nft_mut_ref = object::borrow(nft_object_ref);
        type_table::contains<V>(&nft_mut_ref.extend)
    }

    // view

    public fun get_name<T>(objectId: ObjectID, ctx: &Context): String {
        assert_nft_exist_of_id<T>(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT<T>>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.name
    }

    public fun get_uri<T>(objectId: ObjectID, ctx: &Context): String {
        assert_nft_exist_of_id<T>(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT<T>>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.uri
    }

    public fun get_collection<T>(objectId: ObjectID, ctx: &Context): ObjectID {
        assert_nft_exist_of_id<T>(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT<T>>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.collection
    }

    public fun get_creator<T>(objectId: ObjectID, ctx: &Context): address {
        assert_nft_exist_of_id<T>(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT<T>>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.creator
    }

    #[test_only]
    struct Test has key {}

    #[test(sender = @creator)]
    public fun test_create_nft (sender: address){
        let storage_context = context::new_test_context(sender);
        let ctx = &mut storage_context;
        account::create_account_for_test(ctx, sender);

        let (
            collection_object_ref,
            collection_display_object_ref,
            nft_display_object_ref
        ) = create_collection<Test>(
            string::utf8(b"name"),
            string::utf8(b"uri"),
            sender,
            string::utf8(b"description"),
            option::none(),
            ctx
        );

        let collection_mutator_ref = collection::generate_mutator_ref(&collection_object_ref);

        display::set(&mut collection_display_object_ref, string::utf8(b"name"), string::utf8(b"{ name }"));
        display::set(&mut collection_display_object_ref, string::utf8(b"uri"), string::utf8(b"{ uri }"));
        display::set(&mut collection_display_object_ref, string::utf8(b"description"), string::utf8(b"{ description }"));
        display::set(&mut collection_display_object_ref, string::utf8(b"creator"), string::utf8(b"{ creator }"));
        display::set(&mut collection_display_object_ref, string::utf8(b"supply"), string::utf8(b"{ supply }"));

        display::set(&mut nft_display_object_ref, string::utf8(b"name"), string::utf8(b"{ name }"));
        display::set(&mut nft_display_object_ref, string::utf8(b"uri"), string::utf8(b"{ uri }"));

        let nft_object_ref = mint<Test>(
            string::utf8(b"name"),
            string::utf8(b"uri"),
            &collection_mutator_ref,
            sender,
             ctx
        );

        let nft_mutaor_ref = generate_mutator_ref(&nft_object_ref);

        let burner_ref = generate_burner_ref(&nft_object_ref);

        burn(&burner_ref, &collection_mutator_ref,  ctx);

        collection::destroy_mutator_ref(collection_mutator_ref);

        destroy_mutator_ref(nft_mutaor_ref);
        destroy_burner_ref(burner_ref);

        context::drop_test_context(storage_context);
    }

}