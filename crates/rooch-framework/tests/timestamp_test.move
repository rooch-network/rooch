#[test_only]
module rooch_framework::timestamp_test{

    use rooch_framework::timestamp;
    
    #[test]
    fun test_timestamp(){
        rooch_framework::genesis::init_for_test();
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