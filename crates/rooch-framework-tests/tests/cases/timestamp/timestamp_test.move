//# init --addresses test=0x42

//check the timestamp object id
//# run --signers test
script {
    fun main() {
        let object_id = moveos_std::object::named_object_id<moveos_std::timestamp::Timestamp>();
        std::debug::print(&object_id);
    }
}

//Timestamp object as argument.
//# run --signers test --args object:0x3a7dfe7a9a5cd608810b5ebd60c7adf7316667b17ad5ae703af301b74310bcca
script {
    use moveos_std::object::{Self, Object};
    use moveos_std::timestamp::{Self, Timestamp};

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

