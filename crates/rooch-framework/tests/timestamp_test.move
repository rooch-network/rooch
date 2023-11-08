#[test_only]
module rooch_framework::timestamp_test{

    use moveos_std::context;
    use rooch_framework::timestamp;
    
    #[test]
    fun test_timestamp(){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        {
            let timestamp = timestamp::timestamp(&genesis_ctx);
            assert!(timestamp::milliseconds(timestamp) == 0, 1);
        };
        let seconds:u64 = 1;
        timestamp::fast_forward_seconds_for_test(&mut genesis_ctx, seconds);
        assert!(timestamp::now_milliseconds(&genesis_ctx) == timestamp::seconds_to_milliseconds(seconds), 2);
        assert!(timestamp::now_seconds(&genesis_ctx) == seconds, 3);
        {
            let timestamp = timestamp::timestamp(&genesis_ctx);
            assert!(timestamp::milliseconds(timestamp) == timestamp::seconds_to_milliseconds(seconds), 4);
        };
        context::drop_test_context(genesis_ctx); 
    }
}