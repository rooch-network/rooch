// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::state_reset_tests {
    use std::option;
    use moveos_std::account;
    use rooch_framework::address_mapping;
    use rooch_framework::bitcoin_address;
    use rooch_framework::genesis;
    use rooch_framework::onchain_config;

    #[test]
    fun test_reset_rooch_to_bitcoin_mapping() {
        genesis::init_for_test();
        let btc_addr = bitcoin_address::from_string(&std::string::utf8(
            b"bc1p8xpjpkc9uzj2dexcxjg9sw8lxje85xa4070zpcys589e3rf6k20qm6gjrt",
        ));
        address_mapping::bind_bitcoin_address(btc_addr);
        let rooch_addr = bitcoin_address::to_rooch_address(&btc_addr);
        assert!(option::is_some(&address_mapping::resolve_bitcoin(rooch_addr)), 1);
        let admin = account::create_account_for_testing(onchain_config::admin());
        address_mapping::reset_rooch_to_bitcoin_mapping(&admin);
        assert!(option::is_none(&address_mapping::resolve_bitcoin(rooch_addr)), 2);
        assert!(address_mapping::rooch_to_bitcoin_field_size_for_test() == 0, 3);
    }
}
