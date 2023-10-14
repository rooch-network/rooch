// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::nft {
    use std::string::String;
    use moveos_std::object_ref;
    use moveos_std::object;
    use rooch_framework::collection;
    use moveos_std::object_ref::ObjectRef;
    use moveos_std::context::Context;
    use moveos_std::context;
    use moveos_std::type_table;
    use moveos_std::object::ObjectID;
    use moveos_std::type_table::TypeTable;

    const ENftNotExist: u64 = 100;
    const EMutatorNotExist: u64 = 101;

    struct NFT has store {
        name: String,
        uri: String,
        collection: ObjectID,
        creator: address,
        extend: TypeTable
    }

    struct MutatorRef has store{
        nft: ObjectID,
    }

    struct BurnerRef has store{
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

    #[private_generics(T)]
    public fun add_nft_extend<V: key>(mutator: &ObjectRef<MutatorRef>,val: V,ctx: &mut Context) {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::add( &mut nft_mut_ref.extend, val);
    }


    public fun borrow_nft_extend<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &Context): &V {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_ref = context::borrow_object<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow(nft_object_ref);
        type_table::borrow(&nft_mut_ref.extend)
    }

    #[private_generics(T)]
    public fun borrow_mut_nft_extend<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &mut Context): &mut V {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::borrow_mut(&mut nft_mut_ref.extend)
    }

    #[private_generics(T)]
    public fun remove_nft_extend<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &mut Context) {
        assert_mutator_exist_of_ref(mutator);
        let mutator_object_ref = object_ref::borrow(mutator);
        assert_nft_exist_of_id(mutator_object_ref.nft, ctx);
        let nft_object_mut_ref = context::borrow_object_mut<NFT>(ctx, mutator_object_ref.nft);
        let nft_mut_ref = object::borrow_mut(nft_object_mut_ref);
        type_table::remove(&mut nft_mut_ref.extend)
    }

    public fun contains_nft_extend<V: key>(mutator: &ObjectRef<MutatorRef>,ctx: &Context): bool {
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


}