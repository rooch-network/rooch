module timestamp::timestamp {
    // use std::debug;
    // use moveos_std::context::Context;
    // use rooch_framework::timestamp::{Self, Timestamp};

    use moveos_std::context;
    use moveos_std::context::Context;
    use rooch_framework::timestamp;
    use rooch_framework::timestamp::Timestamp;
    use moveos_std::object::Object;
    use moveos_std::object;

    // struct MyTimeStamp {
    //     stamp_value: timestamp::Timestamp
    // }
    fun init(ctx: &mut Context, account: &signer) {
        // genesis_init
    }

    entry fun time() {
        let object_id = moveos_std::object::singleton_object_id<rooch_framework::timestamp::Timestamp>();
        std::debug::print(&object_id);
    }

    entry fun test1() {}

    entry fun time2(timestamp_obj: &Object<Timestamp>) {
        let timestamp = object::borrow(timestamp_obj);
        let now_secounds = timestamp::seconds(timestamp);
        std::debug::print(&now_secounds);
    }

    entry fun test3(ctx: &Context) {
        let now_secounds = timestamp::now_seconds(ctx);
        std::debug::print(&now_secounds);
    }

    fun test4(ctx: &mut Context, timestamp_obj: &Object<Timestamp>) {
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
