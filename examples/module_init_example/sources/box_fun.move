module rooch_examples::box_fun {
    use std::string::{Self, String};
    use moveos_std::object::ObjectID;
    use moveos_std::storage_context::{StorageContext};
    use rooch_examples::box;
    use rooch_examples::box_friend;
    use std::debug;

    struct Container has key {
        name: String,
        count: u128,
    }

    // for test
    fun init(_ctx: &mut StorageContext) {
        debug::print<String>(&string::utf8(b"module box_fun init finish"));
    }

    public entry fun create_box(
        stoage_ctx: &mut StorageContext,
        name: String,
        count: u128,
    ) {
        let obj = box::create_box(stoage_ctx, name, count);
        box::add_box(stoage_ctx, obj);
    }

    public entry fun remove_box_and_update(
        storage_ctx: &mut StorageContext,
        object_id: ObjectID,
    ) {
        let obj = box::remove_box(storage_ctx, object_id);
        let update_obj = box_friend::change_box(obj);
        box::add_box(storage_ctx, update_obj);
    }
}
