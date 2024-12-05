module moveos_std::gas_schedule {
    use std::vector;
    use std::string::String;
    use moveos_std::bcs;
    use moveos_std::tx_context;
    use moveos_std::object;
    use moveos_std::core_addresses;

    friend moveos_std::genesis;
    
    const ErrorInvalidGasScheduleEntries: u64 = 1;

    /// The initial max gas amount from genesis.
    const InitialMaxGasAmount: u64 = 1_000_000_000u64;
    public fun initial_max_gas_amount(): u64 {
        InitialMaxGasAmount
    }
    /// The max gas amount of the transaction.
    /// This const can be changed via framework upgrade.
    /// We use const other than the GasScheduleConfig for the performance.
    const MaxGasAmount: u64 = 1_000_000_000u64;
    public fun max_gas_amount(): u64 {
        MaxGasAmount
    }

    struct GasScheduleUpdated has store, copy, drop {
        last_updated: u64
    }

    #[data_struct]
    struct GasEntry has store, copy, drop {
        key: String,
        val: u64,
    }

    struct GasSchedule has key {
        schedule_version: u64,
        max_gas_amount: u64,
        entries: vector<GasEntry>,
    }

    #[data_struct]
    struct GasScheduleConfig has copy, drop, store{
        max_gas_amount: u64,
        entries: vector<GasEntry>,
    }

    public(friend) fun genesis_init(gas_schedule_config: GasScheduleConfig){

        let gas_schedule = GasSchedule {
            schedule_version: 0,
            max_gas_amount: gas_schedule_config.max_gas_amount,
            entries: gas_schedule_config.entries,
        };

        let obj = object::new_named_object(gas_schedule);
        object::transfer_extend(obj, @moveos_std);
    }

    public fun new_gas_schedule_config(max_gas_amount: u64, entries: vector<GasEntry>): GasScheduleConfig {
        GasScheduleConfig { max_gas_amount, entries}
    }

    public fun new_gas_entry(key: String, val: u64): GasEntry {
        GasEntry {key, val}
    }

    public fun update_gas_schedule(account: &signer, gas_schedule_config: vector<u8>) {
        core_addresses::assert_system_reserved(account);
        assert!(vector::length(&gas_schedule_config) > 0, ErrorInvalidGasScheduleEntries);

        let gas_schedule_config = bcs::from_bytes<GasScheduleConfig>(gas_schedule_config);
        update_gas_schedule_interanl(gas_schedule_config);
    }

    fun update_gas_schedule_interanl(gas_schedule_config: GasScheduleConfig) {

        let object_id = object::named_object_id<GasSchedule>();
        let obj = object::borrow_mut_object_extend<GasSchedule>(object_id);
        let gas_schedule = object::borrow_mut(obj);

        gas_schedule.schedule_version = gas_schedule.schedule_version + 1;
        gas_schedule.max_gas_amount = gas_schedule_config.max_gas_amount;
        gas_schedule.entries = gas_schedule_config.entries;

        let system = moveos_std::signer::module_signer<GasScheduleUpdated>();
        tx_context::add_attribute_via_system(&system, GasScheduleUpdated {last_updated: 1});
    }

    public fun gas_schedule(): &GasSchedule {
        let object_id = object::named_object_id<GasSchedule>();
        let obj = object::borrow_object<GasSchedule>(object_id);
        object::borrow(obj)
    }

    ///This function will deprecated in the future, please use `max_gas_amount()` instead.
    public fun gas_schedule_max_gas_amount(schedule: &GasSchedule): u64 {
        schedule.max_gas_amount
    }

    public fun gas_schedule_version(schedule: &GasSchedule): u64 {
        schedule.schedule_version
    }

    public fun gas_schedule_entries(schedule: &GasSchedule): &vector<GasEntry> {
        &schedule.entries
    }

    #[test_only]
    public fun update_gas_schedule_for_testing(gas_schedule_config: GasScheduleConfig) {
        update_gas_schedule_interanl(gas_schedule_config);
    }

    #[test]
    fun test_gas_schedule() {
        let gas_config = new_gas_schedule_config(initial_max_gas_amount(), vector::empty());
        genesis_init(gas_config);
        let gas_schedule = gas_schedule();
        assert!(vector::length(gas_schedule_entries(gas_schedule)) == 0, 1000);
        let entries = vector::empty();
        vector::push_back(&mut entries, new_gas_entry(std::string::utf8(b"test1"), 1));
        let gas_schedule_config = new_gas_schedule_config(initial_max_gas_amount(), entries);
        update_gas_schedule_for_testing(gas_schedule_config);
        let gas_schedule2 = gas_schedule();
        let entries2 = gas_schedule_entries(gas_schedule2);
        assert!(vector::length(entries2) == 1, 1002);
    }
}