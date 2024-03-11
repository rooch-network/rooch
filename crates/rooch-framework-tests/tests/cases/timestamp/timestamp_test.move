//# init --addresses test=0x42

//check the timestamp object id
//# run --signers test
script {
    fun main() {
        let object_id = moveos_std::object_id::named_object_id<rooch_framework::timestamp::Timestamp>();
        std::debug::print(&object_id);
    }
}

//Timestamp object as argument.
//# run --signers test --args @0x711ab0301fd517b135b88f57e84f254c94758998a602596be8ae7ba56a0d14b3
script {
    use moveos_std::object::{Self, Object};
    use rooch_framework::timestamp::{Self, Timestamp};

    fun main(timestamp_obj: &Object<Timestamp>) {
        let timestamp = object::borrow(timestamp_obj);
        let now_seconds = timestamp::seconds(timestamp);
        std::debug::print(&now_seconds);
    }
}

//Get timestamp from context
//# run --signers test
script {
    use moveos_std::context::Context;
    use rooch_framework::timestamp;

    fun main(ctx: &Context) {
        let now_seconds = timestamp::now_seconds(ctx);
        std::debug::print(&now_seconds);
    }
}

// Update timestamp
//# run --signers test
script {
    use moveos_std::context::Context;
    use rooch_framework::timestamp;

    fun main(ctx: &mut Context) {
        let seconds = 100;
        timestamp::fast_forward_seconds_for_local(ctx, seconds);
        let seconds_from_ctx = timestamp::now_seconds(ctx);
        assert!(seconds == seconds_from_ctx, 1);
    }
}

