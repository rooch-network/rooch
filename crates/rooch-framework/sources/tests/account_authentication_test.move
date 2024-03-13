// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the account authentication module.
/// Migrate the tests from the account_authentication module to this module for avoid cyclic dependencies.
module rooch_framework::account_authentication_test{
    use rooch_framework::auth_validator_registry;
    use rooch_framework::account_authentication::{install_auth_validator, is_auth_validator_installed};

    #[test_only]
    struct TestAuthValidator has store{
    }
    #[test]
    fun test_install_auth_validator(){
        rooch_framework::genesis::init_for_test();
            
        let user_address = @0x42;
        let user_signer = moveos_std::account::create_signer_for_testing(user_address);
        
        let validator_id = auth_validator_registry::register<TestAuthValidator>();

        install_auth_validator<TestAuthValidator>(&user_signer);
        
        assert!(is_auth_validator_installed(user_address, validator_id), 1000);
        assert!(!is_auth_validator_installed(user_address, 100000), 1001);

        
        
    }
}
