//# init --addresses test=0x42

//check the timestamp object id
//# run --signers test
script {
    fun main() {
        let object_id = moveos_std::object::named_object_id<moveos_std::object::Timestamp>();
        std::debug::print(&object_id);
    }
}

//Timestamp object as argument.
//# run --signers test --args object:0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9
script {
    use moveos_std::object::{Self, Object, Timestamp};
    use moveos_std::timestamp::{Self};

    fun main(timestamp_obj: &Object<Timestamp>) {
        let timestamp = object::borrow(timestamp_obj);
        let now_seconds = timestamp::seconds(timestamp);
        std::debug::print(&now_seconds);
    }
}

//Get timestamp from context
//# run --signers test
script {
    
    use moveos_std::timestamp;

    fun main() {
        let now_seconds = timestamp::now_seconds();
        std::debug::print(&now_seconds);
    }
}

// Update timestamp
//# run --signers test
script {
    
    use rooch_framework::timestamp;

    fun main() {
        let seconds = 100;
        timestamp::fast_forward_seconds_for_local(seconds);
        let seconds_from_ctx = moveos_std::timestamp::now_seconds();
        assert!(seconds == seconds_from_ctx, 1);
    }
}

