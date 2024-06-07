// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module moveos_std::timestamp_test{
    use moveos_std::timestamp;
    
    #[test]
    fun test_timestamp(){
        moveos_std::genesis::init_for_test();
        {
            let timestamp = timestamp::timestamp();
            assert!(timestamp::milliseconds(timestamp) == 0, 1);
        };
        let seconds:u64 = 1;
        timestamp::fast_forward_seconds_for_test(seconds);
        assert!(timestamp::now_milliseconds() == timestamp::seconds_to_milliseconds(seconds), 2);
        assert!(timestamp::now_seconds() == seconds, 3);
        {
            let timestamp = timestamp::timestamp();
            assert!(timestamp::milliseconds(timestamp) == timestamp::seconds_to_milliseconds(seconds), 4);
        };
    }
}
