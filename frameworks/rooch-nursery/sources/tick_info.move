// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::tick_info {

    use std::option::{Self, Option};
    use std::string::{Self, String};
    use std::vector;
    
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::type_info;
    use moveos_std::tx_context;
    use moveos_std::result::{Result, ok, err};
    use moveos_std::string_utils;
    
    use bitcoin_move::ord::{Self, InscriptionID};
    use rooch_nursery::bitseed::{Self, Bitseed};

    const ErrorMetaprotocolNotFound: u64 = 1;
    const ErrorTickNotFound: u64 = 2;
    const ErrorNoMintFactory: u64 = 3;
    const ErrorInvalidMintFactory: u64 = 4;
    const ErrorMaxSupplyReached: u64 = 5;

    friend rooch_nursery::inscribe_factory;
    friend rooch_nursery::genesis;

    const BIT_SEED_GENERATOR_TICK: vector<u8> = b"generator";

    /// Store the tick -> TickInfo ObjectID mapping in Object<TickInfoStore> dynamic fields.
    struct TickInfoStore has key {
    }

    struct TickInfo has key {
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
        has_user_input: bool,
        /// The generator or factory deploy arguments.
        deploy_args: Option<vector<u8>>,
        /// The current supply of the tick.
        supply: u64,
    }

    public(friend) fun genesis_init(){
        // init built-in generator tick
        let tick = string::utf8(BIT_SEED_GENERATOR_TICK);
        let repeat = 0;
        let has_user_input = false;
        //u64max
        let max = 18_446_744_073_709_551_615u64;
        
        deploy_tick(bitseed::metaprotocol(), tick, option::none(), option::none(), max, repeat, has_user_input, option::none());
        let module_signer = moveos_std::signer::module_signer<TickInfoStore>();
        ord::register_metaprotocol_via_system<Bitseed>(&module_signer, bitseed::metaprotocol());
    }
    
    /// Check if the tick is deployed.
    public fun is_deployed(metaprotocol: String, tick: String) : bool {
        let tick = string_utils::to_lower_case(&tick);
        let store_id = object::custom_object_id<String, TickInfoStore>(metaprotocol);
        if (!object::exists_object(store_id)){
            return false
        };
        let store = borrow_tick_info_store(metaprotocol);
        object::contains_field(store, tick)
    }


    public fun borrow_tick_info(metaprotocol: String, tick: String) : &TickInfo {
        let tick = string_utils::to_lower_case(&tick);
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

    fun borrow_mut_or_create_tick_info_store(metaprotocol: String) : &mut Object<TickInfoStore>{
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
        has_user_input: bool,
        deploy_args: Option<vector<u8>>,
    ) : ObjectID {
        let tick = string_utils::to_lower_case(&tick);
        let store = borrow_mut_or_create_tick_info_store(metaprotocol);
        let tick_info = TickInfo {
            metaprotocol,
            tick,
            generator,
            factory,
            max,
            repeat,
            has_user_input,
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
        let tick = string_utils::to_lower_case(&tick);
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
        let bitseed = bitseed::new(metaprotocol, tick, bid, real_amount, option::none(), vector::empty());
        tick_info.supply = tick_info.supply + amount;
        bitseed
    }

    public(friend) fun mint_on_bitcoin(metaprotocol: String, tick: String, amount: u64) : Result<Object<Bitseed>>{
        let tick = string_utils::to_lower_case(&tick);
        let tick_info = borrow_mut_tick_info(metaprotocol, tick);
        if (tick_info.supply + amount > tick_info.max){
            return err(b"maximum supply exceeded")
        };
        let bid = tx_context::fresh_address();
        let bitseed = bitseed::new(metaprotocol, tick, bid, amount, option::none(), vector::empty());
        ok(bitseed)
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

    public fun supply(tick_info: &TickInfo) : u64 {
        tick_info.supply
    }

    public fun repeat(tick_info: &TickInfo) : u64 {
        tick_info.repeat
    }

    public fun has_user_input(tick_info: &TickInfo) : bool {
        tick_info.has_user_input
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
        deploy_tick(metaprotocol, tick, generator, factory, max, repeat, false, deploy_args)
    }

    #[test_only]
    public fun init_for_testing() {
        Self::genesis_init()
    }
}