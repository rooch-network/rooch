// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::tick_info {

    use std::option::{Self, Option};
    use std::string::String;
    use std::vector;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::type_info;
    use moveos_std::tx_context;
    use bitcoin_move::ord::{InscriptionID};
    use rooch_nursery::bitseed_on_l2::{Self, Bitseed};

    const ErrorMetaprotocolNotFound: u64 = 1;
    const ErrorTickNotFound: u64 = 2;
    const ErrorNoMintFactory: u64 = 3;
    const ErrorInvalidMintFactory: u64 = 4;
    const ErrorMaxSupplyReached: u64 = 5;

    friend rooch_nursery::bitseed;

    /// Store the tick -> TickInfo ObjectID mapping in Object<TickInfoStore> dynamic fields.
    struct TickInfoStore has key {
    }

    struct TickInfo has key{
        /// The metaprotocol of the tick.
        metaprotocol: String,
        /// The tick name
        tick: String,
        /// The generator of the tick on Bitcoin.
        /// If generator is Some, means the tick is issued on Bitcoin.
        generator: Option<InscriptionID>,
        /// The mint factory of the tick on Rooch
        /// If mint factory is Some, means the tick is issued on Rooch.
        factory: Option<String>,
        /// The max supply of the tick.
        max: u64,
        /// The bitseed attributes can be repeated count.
        // TODO should we support repeat?
        repeat: u64,
        //TODO the has_user_input should be the generator's attribute. 
        //has_user_input: bool,
        /// The generator or factory deploy arguments.
        deploy_args: Option<vector<u8>>,
        /// The current supply of the tick.
        supply: u64,
    }

    public fun borrow_tick_info(metaprotocol: String, tick: String) : &TickInfo {
        let store = borrow_tick_info_store(metaprotocol);
        let tick_info_obj_id: ObjectID = *object::borrow_field(store, tick);
        let tick_info_obj = object::borrow_object(tick_info_obj_id);
        object::borrow(tick_info_obj)
    }

    fun borrow_mut_tick_info(metaprotocol: String, tick: String) : &mut TickInfo {
        let store = borrow_tick_info_store(metaprotocol);
        let tick_info_obj_id: ObjectID = *object::borrow_field(store, tick);
        let tick_info_obj = object::borrow_mut_object_shared(tick_info_obj_id);
        object::borrow_mut(tick_info_obj)
    }

    fun borrow_tick_info_store(metaprotocol: String) : &Object<TickInfoStore>{
        let store_id = object::custom_object_id<String, TickInfoStore>(metaprotocol);
        assert!(object::exists_object(store_id), ErrorMetaprotocolNotFound);
        object::borrow_object<TickInfoStore>(store_id)
    }

    fun borrow_mut_tick_info_store(metaprotocol: String) : &mut Object<TickInfoStore>{
        let store_id = object::custom_object_id<String, TickInfoStore>(metaprotocol);
        if (!object::exists_object(store_id)){
            let store = TickInfoStore{};
            let store_obj = object::new_with_id(metaprotocol, store);
            object::to_shared(store_obj);
        };
        object::borrow_mut_object_shared<TickInfoStore>(store_id)
    }

    
    public(friend) fun deploy_tick(
        metaprotocol: String,
        tick: String,
        generator: Option<InscriptionID>,
        factory: Option<String>,
        max: u64,
        repeat: u64,
        deploy_args: Option<vector<u8>>,
    ) : ObjectID {
        let store = borrow_mut_tick_info_store(metaprotocol);
        let tick_info = TickInfo {
            metaprotocol,
            tick,
            generator,
            factory,
            max,
            repeat,
            deploy_args,
            supply: 0,
        };
        let tick_info_obj = object::new(tick_info);
        let tick_info_obj_id = object::id(&tick_info_obj);
        object::add_field(store, tick, tick_info_obj_id);
        object::to_shared(tick_info_obj);
        tick_info_obj_id
    }

    #[private_generics(F)]
    public fun mint<F>(metaprotocol: String, tick: String, amount: u64) : Object<Bitseed>{
        let factory_type = type_info::type_name<F>();
        let tick_info = borrow_mut_tick_info(metaprotocol, tick);
        assert!(option::is_some(&tick_info.factory), ErrorNoMintFactory);
        let factory = option::destroy_some(tick_info.factory);
        assert!(factory == factory_type, ErrorInvalidMintFactory);
        assert!(tick_info.supply < tick_info.max, ErrorMaxSupplyReached);
        let real_amount = if (tick_info.supply + amount > tick_info.max) {
            tick_info.max - tick_info.supply
        } else {
            amount
        };
        let bid = tx_context::fresh_address();
        let bitseed = bitseed_on_l2::new(metaprotocol, tick, bid, real_amount, option::none(), vector::empty());
        tick_info.supply = tick_info.supply + amount;
        bitseed
    }

    // ================== TickInfo Get functions ==========================

    public fun metaprotocol(tick_info: &TickInfo) : String {
        tick_info.metaprotocol
    }

    public fun tick(tick_info: &TickInfo) : String {
        tick_info.tick
    }

    public fun generator(tick_info: &TickInfo) : Option<InscriptionID> {
        tick_info.generator
    }

    public fun factory(tick_info: &TickInfo) : Option<String> {
        tick_info.factory
    }

    public fun max(tick_info: &TickInfo) : u64 {
        tick_info.max
    }

    public fun deploy_args(tick_info: &TickInfo) : Option<vector<u8>> {
        tick_info.deploy_args
    }

    #[test_only]
    public fun deploy_for_testing(
        metaprotocol: String,
        tick: String,
        generator: Option<InscriptionID>,
        factory: Option<String>,
        max: u64,
        repeat: u64,
        deploy_args: Option<vector<u8>>,
    ) : ObjectID {
        std::debug::print(&factory);
        deploy_tick(metaprotocol, tick, generator, factory, max, repeat, deploy_args)
    }
}