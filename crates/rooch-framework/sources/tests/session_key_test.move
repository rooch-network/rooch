#[test_only]
/// This test module is used to test the session key
module rooch_framework::session_key_test{

    use std::vector;
    use std::option;
    use moveos_std::signer;
    use moveos_std::context;
    use moveos_std::bcs;
    use rooch_framework::session_key;
    use rooch_framework::timestamp;

    #[test(sender=@0x42)]
    fun test_session_key_end_to_end(sender:&signer){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        let sender_addr = signer::address_of(sender);
        let user_ctx = context::new_test_context(sender_addr);
        let scope = session_key::new_session_scope(@0x1, std::ascii::string(b"*"), std::ascii::string(b"*"));
        let authentication_key = bcs::to_bytes(&sender_addr);
        let max_inactive_interval = 10;
        session_key::create_session_key(&mut user_ctx, sender, authentication_key, vector::singleton(scope), max_inactive_interval);
        let session_key_opt = session_key::get_session_key(&user_ctx, sender_addr, authentication_key);
        assert!(option::is_some(&session_key_opt), 1000);

        assert!(!session_key::is_expired_session_key(&mut user_ctx, sender_addr, authentication_key), 1001);
        timestamp::fast_forward_seconds_for_test(&mut user_ctx, 9);
        assert!(!session_key::is_expired_session_key(&mut user_ctx, sender_addr, authentication_key), 1002);
        session_key::active_session_key_for_test(&mut user_ctx, authentication_key);
        timestamp::fast_forward_seconds_for_test(&mut user_ctx, 9);
        assert!(!session_key::is_expired_session_key(&mut user_ctx, sender_addr, authentication_key), 1003);
        timestamp::fast_forward_seconds_for_test(&mut user_ctx, 2);
        assert!(session_key::is_expired_session_key(&mut user_ctx, sender_addr, authentication_key), 1004);
        session_key::remove_session_key(&mut user_ctx, sender, authentication_key);

        context::drop_test_context(user_ctx);
        context::drop_test_context(genesis_ctx);
    }

}