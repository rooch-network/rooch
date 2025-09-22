// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module handles transaction gas payment with smart fallback between account store and payment hub.
/// It provides a clean separation of concerns for gas payment logic.
module rooch_framework::transaction_gas {
    use std::option;
    use moveos_std::tx_context;
    use rooch_framework::gas_coin::{Self, RGas};
    use rooch_framework::coin::{Self, Coin};
    use rooch_framework::payment_channel;

    friend rooch_framework::transaction_validator;

    /// Gas usage information for proper refund tracking
    struct GasUsageInfo has copy, drop, store {
        hub_amount_used: u256,
        account_amount_used: u256,
    }

    /// Enhanced gas deduction that tries payment hub first, then account store
    /// Returns: (gas_coin, usage_info)
    public(friend) fun deduct_transaction_gas(addr: address, amount: u256): (Coin<RGas>, GasUsageInfo) {
        let hub_balance = payment_channel::get_balance_in_hub<RGas>(addr);
        
        if (hub_balance >= amount) {
            // Sufficient in payment hub, use it entirely
            let gas_coin = withdraw_rgas_from_hub(addr, amount);
            let usage_info = GasUsageInfo {
                hub_amount_used: amount,
                account_amount_used: 0u256,
            };
            (gas_coin, usage_info)
        } else if (hub_balance > 0) {
            // Partial from hub, rest from account store
            let hub_coin = withdraw_rgas_from_hub(addr, hub_balance);
            let remaining = amount - hub_balance;
            let account_coin = gas_coin::deduct_gas(addr, remaining);
            
            coin::merge(&mut hub_coin, account_coin);
            let usage_info = GasUsageInfo {
                hub_amount_used: hub_balance,
                account_amount_used: remaining,
            };
            (hub_coin, usage_info)
        } else {
            // No hub balance, use account store entirely
            let gas_coin = gas_coin::deduct_gas(addr, amount);
            let usage_info = GasUsageInfo {
                hub_amount_used: 0u256,
                account_amount_used: amount,
            };
            (gas_coin, usage_info)
        }
    }

    /// Refund remaining gas proportionally to original sources
    public(friend) fun refund_transaction_gas(addr: address, remaining_gas: Coin<RGas>, usage_info: GasUsageInfo) {
        let remaining_amount = coin::value(&remaining_gas);
        
        if (remaining_amount == 0) {
            coin::destroy_zero(remaining_gas);
            return
        };
        
        let total_used = usage_info.hub_amount_used + usage_info.account_amount_used;
        if (total_used == 0) {
            // Shouldn't happen, but handle gracefully
            gas_coin::refund_gas(addr, remaining_gas);
            return
        };
        
        // Calculate proportional refund
        let to_hub = remaining_amount * usage_info.hub_amount_used / total_used;
        let _to_account = remaining_amount - to_hub;
        
        // Refund to payment hub
        if (to_hub > 0) {
            let hub_coin = coin::extract(&mut remaining_gas, to_hub);
            deposit_rgas_to_hub(addr, hub_coin);
        };
        
        // Refund to account store
        if (coin::value(&remaining_gas) > 0) {
            gas_coin::refund_gas(addr, remaining_gas);
        } else {
            coin::destroy_zero(remaining_gas);
        }
    }

    /// Check total available gas balance across all sources
    public fun total_available_gas_balance(addr: address): u256 {
        gas_coin::balance(addr) + payment_channel::get_balance_in_hub<RGas>(addr)
    }

    /// Store gas usage info in transaction context for later retrieval
    public(friend) fun store_gas_usage_info(usage_info: GasUsageInfo) {
        let system_signer = moveos_std::signer::module_signer<GasUsageInfo>();
        tx_context::add_attribute_via_system(&system_signer, usage_info);
    }

    /// Retrieve gas usage info from transaction context
    public(friend) fun get_gas_usage_info(): option::Option<GasUsageInfo> {
        tx_context::get_attribute<GasUsageInfo>()
    }

    // === Internal Helper Functions ===

    /// Withdraw RGAS from payment hub using friend access
    fun withdraw_rgas_from_hub(addr: address, amount: u256): Coin<RGas> {
        payment_channel::withdraw_from_hub_internal<RGas>(addr, amount)
    }

    /// Deposit RGAS to payment hub using generic method
    fun deposit_rgas_to_hub(addr: address, rgas_coin: Coin<RGas>) {
        payment_channel::deposit_to_hub<RGas>(addr, rgas_coin)
    }

    // === Test Functions ===

    #[test_only]
    /// Test helper for gas deduction
    public fun test_deduct_transaction_gas(addr: address, amount: u256): (Coin<RGas>, GasUsageInfo) {
        deduct_transaction_gas(addr, amount)
    }

    #[test_only]
    /// Test helper for gas refund
    public fun test_refund_transaction_gas(addr: address, remaining_gas: Coin<RGas>, usage_info: GasUsageInfo) {
        refund_transaction_gas(addr, remaining_gas, usage_info)
    }

    #[test]
    /// Test basic transaction gas functionality
    fun test_transaction_gas_basic() {
        use rooch_framework::genesis;
        
        // Initialize for testing
        genesis::init_for_test();
        
        // Create test account
        let account = moveos_std::account::create_signer_for_testing(@0x42);
        let addr = moveos_std::signer::address_of(&account);
        
        // Setup account with gas
        gas_coin::faucet_for_test(addr, 1000000u256);
        
        // Test gas deduction (should use account store since no payment hub)
        let (gas_coin, usage_info) = test_deduct_transaction_gas(addr, 100000u256);
        assert!(coin::value(&gas_coin) == 100000u256, 1);
        assert!(usage_info.hub_amount_used == 0u256, 2);
        assert!(usage_info.account_amount_used == 100000u256, 3);
        
        // Test gas refund
        test_refund_transaction_gas(addr, gas_coin, usage_info);
        
        // Verify balance after refund
        let final_balance = gas_coin::balance(addr);
        assert!(final_balance == 1000000u256, 4);
    }

    #[test]
    /// Test total available gas balance
    fun test_total_available_gas_balance_function() {
        use rooch_framework::genesis;
        
        // Initialize for testing
        genesis::init_for_test();
        
        // Create test account
        let account = moveos_std::account::create_signer_for_testing(@0x43);
        let addr = moveos_std::signer::address_of(&account);
        
        // Setup account with gas
        gas_coin::faucet_for_test(addr, 500000u256);
        
        // Test total balance (should only be account balance since no payment hub)
        let total_balance = total_available_gas_balance(addr);
        assert!(total_balance == 500000u256, 1);
    }
}
