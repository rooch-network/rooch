//# init --addresses test=0x42

//check the timestamp object id
//# run --signers test
script {
    fun main() {
        let object_id = moveos_std::object::singleton_object_id<rooch_framework::timestamp::Timestamp>();
        std::debug::print(&object_id);
    }
}

//Timestamp object as argument.
//# run --signers test --args @0x1e9afd0c19db5da27f8e9a1ad98a551259e51db612dc771a9f78c4142059b391
script {
    use moveos_std::object::{Self, Object};
    use rooch_framework::timestamp::{Self, Timestamp};

    fun main(timestamp_obj: &Object<Timestamp>) {
        let timestamp = object::borrow(timestamp_obj);
        let now_secounds = timestamp::seconds(timestamp);
        std::debug::print(&now_secounds);
    }
}

//Get timestamp from context
//# run --signers test
script {
    use moveos_std::context::Context;
    use rooch_framework::timestamp;

    fun main(ctx: &Context) {
        let now_secounds = timestamp::now_seconds(ctx);
        std::debug::print(&now_secounds);
    }
}

// Update timestamp
//# run --signers test --args @0x1e9afd0c19db5da27f8e9a1ad98a551259e51db612dc771a9f78c4142059b391
script {
    use moveos_std::context::Context;
    use moveos_std::object::{Self, Object};
    use rooch_framework::timestamp::{Self, Timestamp};

    fun main(ctx: &mut Context, timestamp_obj: &Object<Timestamp>) {
        let timestamp = object::borrow(timestamp_obj);
        let secounds_from_arg = timestamp::seconds(timestamp);
        let secounds_from_ctx = timestamp::now_seconds(ctx);
        assert!(secounds_from_arg == secounds_from_ctx, 1);
        let seconds = 100;
        timestamp::fast_forward_seconds_for_local(ctx, seconds);
        let secounds_from_arg = timestamp::seconds(timestamp);
        let secounds_from_ctx = timestamp::now_seconds(ctx);
        assert!(secounds_from_arg == secounds_from_ctx, 2);
        assert!(secounds_from_arg == seconds, 3); 
    }
}

