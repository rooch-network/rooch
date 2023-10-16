// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::nft {
    use std::string::String;
    use rooch_framework::display::Display;
    use moveos_std::object_ref;
    use moveos_std::object;
    use rooch_framework::collection;
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
    use rooch_framework::display;

    const ENftNotExist: u64 = 100;
    const EMutatorNotExist: u64 = 101;

    struct NFT has key {
        name: String,
        uri: String,
        collection: ObjectID,
        creator: address,
        extend: TypeTable
    }

    struct MutatorRef has key {
        nft: ObjectID,
    }

    struct BurnerRef has key {
        nft: ObjectID,
    }

    public fun mint(
        name: String,
        uri: String,
        mutator_ref: &ObjectRef<collection::MutatorRef>,
        creator: address,
        ctx: &mut Context
    ): ObjectRef<NFT> {
        collection::assert_mutator_exist_of_ref(mutator_ref);
        let nft = NFT {
            name,
            uri,
            collection: collection::get_collection_id(mutator_ref),
            creator,
            extend: type_table::new(ctx)
        };

        collection::increment_supply(mutator_ref, ctx);

        let object_ref = context::new_object_with_owner(
            ctx,
            creator,
            nft
        );

        object_ref
    }

    public fun burn (
        burn_ref: &ObjectRef<BurnerRef>,
        mutator_ref: &ObjectRef<collection::MutatorRef>,
        ctx: &mut Context
    ) {
        assert_burner_exist_of_ref(burn_ref);
        let burner_object_ref = object_ref::borrow(burn_ref);
        assert_burner_exist_of_id(burner_object_ref.nft, ctx);
        // let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, burner_object_ref.nft);
        collection::decrement_supply(mutator_ref, ctx);
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
        ) = context::remove_object<NFT>(ctx, burner_object_ref.nft);
        type_table::destroy_empty(extend)
    }

    public fun generate_mutator_ref(nft_object_ref: &ObjectRef<NFT>, ctx: &mut Context):ObjectRef<MutatorRef>{
        let mutator_ref = context::new_object_with_owner(
            ctx,
            object_ref::owner(nft_object_ref),
            MutatorRef {
                nft: object_ref::id(nft_object_ref),
            }
        );
        mutator_ref
    }

    public fun destroy_mutator_ref(mutator_ref :ObjectRef<MutatorRef>):ObjectID{
        assert_mutator_exist_of_ref(&mutator_ref);
        let MutatorRef {
            nft
        } = object_ref::remove<MutatorRef>(mutator_ref);
        nft
    }

    public fun generate_burner_ref(nft_object_ref: &ObjectRef<NFT>, ctx: &mut Context):ObjectRef<BurnerRef>{
        let burner_ref = context::new_object_with_owner(
            ctx,
            object_ref::owner(nft_object_ref),
            BurnerRef {
                nft: object_ref::id(nft_object_ref),
            }
        );
        burner_ref
    }

    public fun destroy_burner_ref(burner_ref :ObjectRef<BurnerRef>):ObjectID{
        assert_burner_exist_of_ref(&burner_ref);
        let BurnerRef {
            nft
        } = object_ref::remove<BurnerRef>(burner_ref);
        nft
    }

    // assert
    public fun assert_nft_exist_of_id(objectId: ObjectID, ctx: &Context) {
        assert!(context::exist_object(ctx, objectId), ENftNotExist);
        context::borrow_object<NFT>(ctx, objectId);
    }

    public fun assert_nft_exist_of_ref(nft_object_ref: &ObjectRef<NFT>) {
        assert!(object_ref::exist_object(nft_object_ref), ENftNotExist);
    }

    public fun assert_mutator_exist_of_ref(mutator_ref: &ObjectRef<MutatorRef>) {
        assert!(object_ref::exist_object(mutator_ref), EMutatorNotExist);
    }

    public fun assert_mutator_exist_of_id(objectId: ObjectID, ctx: &Context) {
        assert!(context::exist_object(ctx, objectId), EMutatorNotExist);
        context::borrow_object<MutatorRef>(ctx, objectId);
    }

    public fun assert_burner_exist_of_ref(burner_ref: &ObjectRef<BurnerRef>) {
        assert!(object_ref::exist_object(burner_ref), EMutatorNotExist);
    }

    public fun assert_burner_exist_of_id(objectId: ObjectID, ctx: &Context) {
        assert!(context::exist_object(ctx, objectId), EMutatorNotExist);
        context::borrow_object<BurnerRef>(ctx, objectId);
    }

    public fun add_display(mutator: &ObjectRef<MutatorRef>, display: Display, ctx: &mut Context){
        add_extend_internal(mutator, display, ctx);
    }

    public fun borrow_display(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):&Display{
        borrow_extend_internal(mutator, ctx)
    }

    public fun borrow_mut_display(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):&mut Display{
        borrow_mut_extend_internal(mutator, ctx)
    }

    public fun remove_display(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):Display{
        remove_extend_internal(mutator, ctx)
    }

    public fun contains_display(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context): bool{
        contains_extend_internal<Display>(mutator, ctx)
    }

    #[private_generics(V)]
    public fun add_extend<V: key>(mutator: &ObjectRef<MutatorRef>, val: V, ctx: &mut Context){
        add_extend_internal(mutator, val, ctx);
    }

    public fun borrow_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):&V{
        borrow_extend_internal(mutator, ctx)
    }

    #[private_generics(V)]
    public fun borrow_mut_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):&mut V{
        borrow_mut_extend_internal(mutator, ctx)
    }

    #[private_generics(V)]
    public fun remove_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context):V{
        remove_extend_internal(mutator, ctx)
    }

    public fun contains_extend<V: key>(mutator: &ObjectRef<MutatorRef>, ctx: &mut Context): bool{
        contains_extend_internal<V>(mutator, ctx)
    }


    fun add_extend_internal<V: key>(mutator: &ObjectRef<MutatorRef>,val: V,ctx: &mut Context) {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::add( &mut nft_mut_ref.extend, val);
    }

    fun borrow_extend_internal<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &Context): &V {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow(nft_object_ref);
        type_table::borrow(&nft_mut_ref.extend)
    }

    fun borrow_mut_extend_internal<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &mut Context): &mut V {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::borrow_mut(&mut nft_mut_ref.extend)
    }

    fun remove_extend_internal<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &mut Context):V {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::remove<V>(&mut nft_mut_ref.extend)
    }

    fun contains_extend_internal<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &Context): bool {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow(nft_object_ref);
        type_table::contains<V>(&nft_mut_ref.extend)
    }

    // view

    public fun get_name(objectId: ObjectID, ctx: &Context): String {
        assert_nft_exist_of_id(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.name
    }

    public fun get_uri(objectId: ObjectID, ctx: &Context): String {
        assert_nft_exist_of_id(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.uri
    }

    public fun get_collection(objectId: ObjectID, ctx: &Context): ObjectID {
        assert_nft_exist_of_id(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.collection
    }

    public fun get_creator(objectId: ObjectID, ctx: &Context): address {
        assert_nft_exist_of_id(objectId, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, objectId);
        let nft = object::borrow(nft_object_ref);
        nft.creator
    }

    #[test(sender = @0x2)]
    public fun test_create_nft (sender: address){
        let ctx = context::new_test_context(sender);

        let collection_object_ref = collection::create_collection(
            string::utf8(b"name"),
            string::utf8(b"uri"),
            sender,
            string::utf8(b"description"),
            option::none(),
            &mut ctx
        );

        let collection_mutator_ref = collection::generate_mutator_ref(&collection_object_ref, &mut ctx);
        let collcetion_display =  display::new();
        display::set(&mut collcetion_display, string::utf8(b"name"), string::utf8(b"{ name }"));
        display::set(&mut collcetion_display, string::utf8(b"uri"), string::utf8(b"{ uri }"));
        display::set(&mut collcetion_display, string::utf8(b"description"), string::utf8(b"{ description }"));
        display::set(&mut collcetion_display, string::utf8(b"creator"), string::utf8(b"{ creator }"));
        display::set(&mut collcetion_display, string::utf8(b"supply"), string::utf8(b"{ supply }"));

        collection::add_display(
            &collection_mutator_ref,
            collcetion_display,
            &mut ctx
        );

        let nft_object_ref = mint(
            string::utf8(b"name"),
            string::utf8(b"uri"),
            &collection_mutator_ref,
            sender,
            &mut ctx
        );

        let nft_display = display::new();
        display::set(&mut nft_display, string::utf8(b"name"), string::utf8(b"{ name }"));
        display::set(&mut nft_display, string::utf8(b"uri"), string::utf8(b"{ uri }/{ name }.png"));
        display::set(&mut nft_display, string::utf8(b"collection"), string::utf8(b"{ collection }"));

        let nft_mutaor_ref = generate_mutator_ref(&nft_object_ref, &mut ctx);
        add_display(
            &nft_mutaor_ref,
            nft_display,
            &mut ctx
        );


        let burner_ref = generate_burner_ref(&nft_object_ref, &mut ctx);
        burn(&burner_ref, &collection_mutator_ref, &mut ctx);


        collection::destroy_mutator_ref(collection_mutator_ref);
        destroy_mutator_ref(nft_mutaor_ref);
        destroy_burner_ref(burner_ref);

        context::drop_test_context(ctx);
    }

}