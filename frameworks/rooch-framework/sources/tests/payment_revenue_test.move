// SPDX-License-Identifier: Apache-2.0
// Copyright (c) Rooch Contributors

#[test_only]
module rooch_framework::payment_revenue_test {
    use std::string;
    use std::option;
    use moveos_std::account;
    use moveos_std::type_info;
    use moveos_std::object;
    use rooch_framework::genesis;
    use rooch_framework::gas_coin::{Self, RGas};
    use rooch_framework::account_coin_store;
    use rooch_framework::coin;
    use rooch_framework::payment_revenue;

    #[test]
    fun test_revenue_hub_creation() {
        genesis::init_for_test();

        let test_addr = @0x42;

        // Initially no revenue hub exists
        assert!(!payment_revenue::revenue_hub_exists(test_addr), 1);
        assert!(payment_revenue::get_revenue_balance<RGas>(test_addr) == 0, 2);

        // Create revenue hub
        let hub_obj = payment_revenue::borrow_or_create_revenue_hub_for_test(test_addr);
        let _hub = object::borrow(hub_obj);

        // Now revenue hub exists
        assert!(payment_revenue::revenue_hub_exists(test_addr), 3);
        assert!(payment_revenue::get_revenue_balance<RGas>(test_addr) == 0, 4);
    }

    #[test]
    fun test_revenue_deposit_and_withdrawal() {
        genesis::init_for_test();

        let test_addr = @0x42;
        let test_account = account::create_signer_for_testing(test_addr);

        // Create some test coins
        let test_coin = gas_coin::mint_for_test(1000u256);
        let generic_coin = coin::convert_coin_to_generic_coin(test_coin);

        // Deposit revenue
        let source = payment_revenue::create_revenue_source(
            string::utf8(b"test_source"),
            option::none(),
            string::utf8(b"Test deposit")
        );
        payment_revenue::deposit_revenue_generic_for_test(test_addr, generic_coin, source);

        // Check balance
        assert!(payment_revenue::get_revenue_balance<RGas>(test_addr) == 1000, 1);
        assert!(payment_revenue::get_revenue_by_source(test_addr, string::utf8(b"test_source"), type_info::type_name<RGas>()) == 1000, 2);

        // Withdraw revenue
        payment_revenue::withdraw_revenue<RGas>(&test_account, 500);

        // Check remaining balance
        assert!(payment_revenue::get_revenue_balance<RGas>(test_addr) == 500, 3);

        // Check account coin store received the withdrawal
        assert!(account_coin_store::balance<RGas>(test_addr) == 500, 4);
    }


    #[test]
    #[expected_failure(abort_code = rooch_framework::payment_revenue::ErrorInsufficientBalance, location = rooch_framework::payment_revenue)]
    fun test_withdraw_insufficient_balance() {
        genesis::init_for_test();

        let test_addr = @0x42;
        let test_account = account::create_signer_for_testing(test_addr);

        // Try to withdraw without any revenue
        payment_revenue::withdraw_revenue<RGas>(&test_account, 100);
    }

    #[test]
    fun test_preview_withdrawal_fee() {
        genesis::init_for_test();

        let test_addr = @0x42;

        // Test fee preview (currently returns zero fee)
        let (gross, fee, net) = payment_revenue::preview_withdrawal_fee<RGas>(test_addr, 1000);
        assert!(gross == 1000, 1);
        assert!(fee == 0, 2);
        assert!(net == 1000, 3);
    }

    #[test]
    fun test_revenue_event_handles_are_per_hub_and_type() {
        genesis::init_for_test();

        let addr1 = @0x42;
        let addr2 = @0x43;

        let hub1 = payment_revenue::borrow_or_create_revenue_hub_for_test(addr1);
        let hub2 = payment_revenue::borrow_or_create_revenue_hub_for_test(addr2);

        let hub1_id = object::id(hub1);
        let hub2_id = object::id(hub2);

        let hub1_deposit_handle = payment_revenue::revenue_event_handle_id_for_test<rooch_framework::payment_revenue::RevenueDepositedEvent>(hub1_id);
        let hub1_deposit_handle_again = payment_revenue::revenue_event_handle_id_for_test<rooch_framework::payment_revenue::RevenueDepositedEvent>(hub1_id);
        let hub1_withdraw_handle = payment_revenue::revenue_event_handle_id_for_test<rooch_framework::payment_revenue::RevenueWithdrawnEvent>(hub1_id);
        let hub2_deposit_handle = payment_revenue::revenue_event_handle_id_for_test<rooch_framework::payment_revenue::RevenueDepositedEvent>(hub2_id);

        // Same hub and event type -> same handle
        assert!(hub1_deposit_handle == hub1_deposit_handle_again, 1);
        // Different hubs -> different handle ids
        assert!(hub1_deposit_handle != hub2_deposit_handle, 2);
        // Different event types on same hub -> different handles
        assert!(hub1_deposit_handle != hub1_withdraw_handle, 3);
    }
}
