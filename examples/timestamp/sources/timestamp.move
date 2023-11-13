module timestamp::timestamp {
    use std::debug::print;
    use rooch_framework::timestamp::{Self, Timestamp};
    use moveos_std::object::Object;
    use moveos_std::object;

    entry fun get_timestamp_object_id() {
        let object_id = object::singleton_object_id<Timestamp>();
        print(&object_id);
    }

    entry fun get_timestamp(timestamp_obj: &Object<Timestamp>) {
        let timestamp = object::borrow(timestamp_obj);
        let now_seconds = timestamp::seconds(timestamp);
        print(&now_seconds);
    }
}
