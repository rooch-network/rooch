module rooch_examples::something_aggregate {
    use moveos_std::object_id::ObjectID;
    use moveos_std::storage_context::StorageContext;
    use rooch_examples::something;
    use rooch_examples::something_do_logic;

    public entry fun create_something(
        stoage_ctx: &mut StorageContext,
        i: u32,
        j: u128,
    ) {
        let obj = something::create_something(stoage_ctx, i, j);
        something::add_something(stoage_ctx, obj);
    }

    public entry fun remove_do_something_add(
        storage_ctx: &mut StorageContext,
        object_id: ObjectID,
    ) {
        let obj = something::remove_something(storage_ctx, object_id);
        let update_obj = something_do_logic::do_something(obj);
        something::add_something(storage_ctx, update_obj);
    }
}
