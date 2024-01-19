module rooch_framework::gas_schedule {
    use std::string::String;
    use moveos_std::object;
    use moveos_std::context;
    use moveos_std::context::Context;
    use moveos_std::bcs;

    friend rooch_framework::genesis;

    #[data_struct]
    struct GasEntry has store, copy, drop {
        key: String,
        val: u64,
    }

    #[data_struct]
    struct GasSchedule has key, copy, drop {
        feature_version: u64,
        entries: vector<GasEntry>,
    }

    public(friend) fun gas_schedule_init(ctx: &mut Context, _genesis_account: &signer, gas_schedule_blob: vector<u8>){
        let gas_schedule = bcs::from_bytes<GasSchedule>(gas_schedule_blob);
        let obj = context::new_named_object(ctx, gas_schedule);
        object::transfer_extend(obj, @rooch_framework);
    }

    public fun get_gas_schedule(ctx: &Context): &GasSchedule {
        let object_id = object::named_object_id<GasSchedule>();
        let obj = context::borrow_object<GasSchedule>(ctx, object_id);
        object::borrow(obj)
    }
}