// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the session key
module rooch_framework::session_key_test{

    use std::vector;
    use std::option;
    use moveos_std::bcs;
    use moveos_std::tx_context;
    use rooch_framework::session_key;
    use rooch_framework::timestamp;

    #[test]
    fun test_session_key_end_to_end(){
        rooch_framework::genesis::init_for_test();
        let sender_addr = tx_context::sender();
        let sender = moveos_std::account::create_signer_for_testing(sender_addr);
        let scope = session_key::new_session_scope(@0x1, std::ascii::string(b"*"), std::ascii::string(b"*"));
        let authentication_key = bcs::to_bytes(&sender_addr);
        let max_inactive_interval = 10;
        session_key::create_session_key(&sender, authentication_key, vector::singleton(scope), max_inactive_interval);
        let session_key_opt = session_key::get_session_key(sender_addr, authentication_key);
        assert!(option::is_some(&session_key_opt), 1000);

        assert!(!session_key::is_expired_session_key(sender_addr, authentication_key), 1001);
        timestamp::fast_forward_seconds_for_test(9);
        assert!(!session_key::is_expired_session_key(sender_addr, authentication_key), 1002);
        session_key::active_session_key_for_test(authentication_key);
        timestamp::fast_forward_seconds_for_test(9);
        assert!(!session_key::is_expired_session_key(sender_addr, authentication_key), 1003);
        timestamp::fast_forward_seconds_for_test(2);
        assert!(session_key::is_expired_session_key(sender_addr, authentication_key), 1004);
        session_key::remove_session_key(&sender, authentication_key);

        
        
    }

}
