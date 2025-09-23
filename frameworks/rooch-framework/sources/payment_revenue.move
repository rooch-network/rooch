// SPDX-License-Identifier: Apache-2.0
// Copyright (c) Rooch Contributors

// Payment Revenue Management for service providers
// Manages revenue earned through payment channels separately from principal funds

module rooch_framework::payment_revenue {
    use std::option::{Option};
    use std::signer;
    use std::string::String;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::table::{Self, Table};
    use moveos_std::type_info;
    use moveos_std::tx_context;
    use moveos_std::event;
    use rooch_framework::coin::{Self, Coin, GenericCoin};
    use rooch_framework::multi_coin_store::{Self, MultiCoinStore};
    use rooch_framework::account_coin_store;

    friend rooch_framework::payment_channel;

    // === Error Constants ===
    /// Insufficient revenue balance for withdrawal.
    const ErrorInsufficientBalance: u64 = 1;
    /// The revenue hub does not exist for this account.
    const ErrorRevenueHubNotExists: u64 = 2;
    /// Invalid revenue source information.
    const ErrorInvalidRevenueSource: u64 = 3;
    /// Withdrawal amount must be greater than zero.
    const ErrorInvalidWithdrawalAmount: u64 = 4;

    // === Structs ===

    /// Revenue source information for tracking and events
    struct RevenueSource has copy, drop, store {
        source_type: String,         // "payment_channel", "staking", etc.
        source_id: Option<ObjectID>, // Specific source ID (e.g., channel_id)
        description: String,         // Optional description
    }

    /// Revenue management hub for each account
    /// Stores revenue earned through various sources separately from principal funds
    struct PaymentRevenueHub has key {
        /// Multi-coin revenue storage
        multi_coin_store: Object<MultiCoinStore>,
        /// Revenue statistics by source type (optional, for detailed display)
        /// source_type -> coin_type -> accumulated_amount
        revenue_by_source: Table<String, Table<String, u256>>,
    }

    // === Events ===

    /// Event emitted when a revenue hub is created
    struct RevenueHubCreatedEvent has copy, drop {
        hub_id: ObjectID,
        owner: address,
    }

    /// Event emitted when revenue is deposited
    struct RevenueDepositedEvent has copy, drop {
        hub_id: ObjectID,
        owner: address,
        coin_type: String,
        amount: u256,
        source_type: String,
        source_id: Option<ObjectID>,
        source_description: String,
    }

    /// Event emitted when revenue is withdrawn
    struct RevenueWithdrawnEvent has copy, drop {
        hub_id: ObjectID,
        owner: address,
        coin_type: String,
        amount: u256,
        // Fee-related fields (reserved for future use)
        fee_amount: u256,    // Currently always 0, will be used when fee mechanism is implemented
        net_amount: u256,    // Currently equals amount, will be amount - fee_amount in future
        fee_rate_bps: u16,   // Currently always 0, will be the fee rate in basis points (0-10000)
    }


    // === Public Functions ===

    /// Get the revenue hub ID for an address
    public fun get_revenue_hub_id(owner: address): ObjectID {
        object::account_named_object_id<PaymentRevenueHub>(owner)
    }

    /// Check if revenue hub exists for an address
    public fun revenue_hub_exists(owner: address): bool {
        let hub_id = get_revenue_hub_id(owner);
        object::exists_object_with_type<PaymentRevenueHub>(hub_id)
    }

    /// Get revenue balance for a specific coin type
    public fun get_revenue_balance<CoinType: key>(owner: address): u256 {
        if (!revenue_hub_exists(owner)) {
            return 0u256
        };
        
        let hub_id = get_revenue_hub_id(owner);
        let hub_obj = object::borrow_object<PaymentRevenueHub>(hub_id);
        let hub = object::borrow(hub_obj);
        let coin_type = type_info::type_name<CoinType>();
        multi_coin_store::balance(&hub.multi_coin_store, coin_type)
    }

    /// Get revenue balance by source type and coin type
    public fun get_revenue_by_source(
        owner: address,
        source_type: String,
        coin_type: String
    ): u256 {
        if (!revenue_hub_exists(owner)) {
            return 0u256
        };
        
        let hub_id = get_revenue_hub_id(owner);
        let hub_obj = object::borrow_object<PaymentRevenueHub>(hub_id);
        let hub = object::borrow(hub_obj);
        
        if (table::contains(&hub.revenue_by_source, source_type)) {
            let source_table = table::borrow(&hub.revenue_by_source, source_type);
            if (table::contains(source_table, coin_type)) {
                *table::borrow(source_table, coin_type)
            } else {
                0u256
            }
        } else {
            0u256
        }
    }

    /// Create a revenue hub for the sender
    public entry fun create_revenue_hub() {
        let sender = tx_context::sender();
        ensure_revenue_hub_exists(sender);
    }

    /// Withdraw revenue to account coin store
    /// Future: This will support fee deduction
    public fun withdraw_revenue<CoinType: key + store>(
        owner: &signer,
        amount: u256,
    ) {
        let owner_addr = signer::address_of(owner);
        let coin_type = type_info::type_name<CoinType>();
        
        assert!(amount > 0, ErrorInvalidWithdrawalAmount);
        assert!(get_revenue_balance<CoinType>(owner_addr) >= amount, ErrorInsufficientBalance);
        
        let hub_obj = borrow_mut_revenue_hub(owner_addr);
        let hub = object::borrow_mut(hub_obj);
        
        // Withdraw from revenue store
        let coin = multi_coin_store::withdraw_by_type<CoinType>(&mut hub.multi_coin_store, amount);
        
        // Deposit to user's account coin store
        account_coin_store::deposit<CoinType>(owner_addr, coin);
        
        // Emit withdrawal event
        let hub_id = object::id(hub_obj);
        event::emit(RevenueWithdrawnEvent {
            hub_id,
            owner: owner_addr,
            coin_type,
            amount,
            fee_amount: 0u256,      // No fees currently implemented
            net_amount: amount,     // Net equals gross when no fees
            fee_rate_bps: 0u16,     // No fee rate currently
        });
    }

    /// Entry function for withdrawing revenue
    public entry fun withdraw_revenue_entry<CoinType: key + store>(
        owner: &signer,
        amount: u256,
    ) {
        withdraw_revenue<CoinType>(owner, amount);
    }

    // === Friend Functions ===

    /// Deposit revenue from trusted modules (friend only)
    public(friend) fun deposit_revenue_generic(
        account: address,
        coin: GenericCoin,
        source: RevenueSource,
    ) {
        let coin_type = coin::coin_type(&coin);
        let amount = coin::generic_coin_value(&coin);
        
        // Ensure revenue hub exists (lazy creation)
        let hub_obj = borrow_or_create_revenue_hub(account);
        let hub = object::borrow_mut(hub_obj);
        
        // Deposit to multi_coin_store
        multi_coin_store::deposit(&mut hub.multi_coin_store, coin);
        
        // Update source statistics
        update_revenue_by_source(hub, &source.source_type, &coin_type, amount);
        
        // Emit deposit event
        let hub_id = object::id(hub_obj);
        event::emit(RevenueDepositedEvent {
            hub_id,
            owner: account,
            coin_type,
            amount,
            source_type: source.source_type,
            source_id: source.source_id,
            source_description: source.description,
        });
    }


    /// Internal withdrawal for system modules (friend only)
    /// Future: This will be used by gas fee deduction or other system operations
    public(friend) fun withdraw_revenue_internal<CoinType: key>(
        account: address,
        amount: u256,
    ): Coin<CoinType> {
        assert!(get_revenue_balance<CoinType>(account) >= amount, ErrorInsufficientBalance);
        
        let hub_obj = borrow_mut_revenue_hub(account);
        let hub = object::borrow_mut(hub_obj);
        
        let generic_coin = multi_coin_store::withdraw(&mut hub.multi_coin_store, type_info::type_name<CoinType>(), amount);
        coin::convert_generic_coin_to_coin<CoinType>(generic_coin)
    }

    // === Helper Functions ===

    /// Ensure revenue hub exists, create if not (lazy creation)
    fun borrow_or_create_revenue_hub(owner: address): &mut Object<PaymentRevenueHub> {
        let hub_obj_id = object::account_named_object_id<PaymentRevenueHub>(owner);
        if (!object::exists_object_with_type<PaymentRevenueHub>(hub_obj_id)) {
            let multi_coin_store = multi_coin_store::create();
            let hub = PaymentRevenueHub {
                multi_coin_store,
                revenue_by_source: table::new(),
            };
            
            // Create account named object
            let hub_obj = object::new_account_named_object(owner, hub);
            object::transfer_extend(hub_obj, owner);
            
            // Emit creation event
            event::emit(RevenueHubCreatedEvent {
                hub_id: hub_obj_id,
                owner,
            });
        };
        object::borrow_mut_object_extend<PaymentRevenueHub>(hub_obj_id)
    }

    /// Ensure revenue hub exists (public version for entry functions)
    fun ensure_revenue_hub_exists(owner: address) {
        let _hub_obj = borrow_or_create_revenue_hub(owner);
    }

    /// Borrow mutable revenue hub (assumes it exists)
    fun borrow_mut_revenue_hub(owner: address): &mut Object<PaymentRevenueHub> {
        let hub_obj_id = object::account_named_object_id<PaymentRevenueHub>(owner);
        assert!(object::exists_object_with_type<PaymentRevenueHub>(hub_obj_id), ErrorRevenueHubNotExists);
        object::borrow_mut_object_extend<PaymentRevenueHub>(hub_obj_id)
    }

    /// Update revenue statistics by source
    fun update_revenue_by_source(
        hub: &mut PaymentRevenueHub,
        source_type: &String,
        coin_type: &String,
        amount: u256,
    ) {
        if (!table::contains(&hub.revenue_by_source, *source_type)) {
            table::add(&mut hub.revenue_by_source, *source_type, table::new<String, u256>());
        };
        
        let source_table = table::borrow_mut(&mut hub.revenue_by_source, *source_type);
        if (table::contains(source_table, *coin_type)) {
            let current_amount = table::borrow_mut(source_table, *coin_type);
            *current_amount = *current_amount + amount;
        } else {
            table::add(source_table, *coin_type, amount);
        };
    }

    // === Future Extension Points ===
    
    /// Preview withdrawal fee (placeholder for future fee mechanism)
    /// Currently returns zero fee, will be implemented when RevenueConfig is added
    public fun preview_withdrawal_fee<CoinType: key>(
        _owner: address,
        amount: u256,
    ): (u256, u256, u256) { // (gross_amount, fee_amount, net_amount)
        // Future: This will calculate actual fees based on RevenueConfig
        (amount, 0u256, amount)
    }

    /// Create a revenue source for tracking purposes
    public fun create_revenue_source(
        source_type: String,
        source_id: Option<ObjectID>,
        description: String,
    ): RevenueSource {
        RevenueSource {
            source_type,
            source_id,
            description,
        }
    }

    #[test_only]
    /// Test helper to create revenue source (alias for testing)
    public fun create_test_revenue_source(
        source_type: String,
        source_id: Option<ObjectID>,
        description: String,
    ): RevenueSource {
        create_revenue_source(source_type, source_id, description)
    }

    #[test_only]
    use rooch_framework::genesis;
    #[test_only]
    use rooch_framework::gas_coin::{Self, RGas};
    #[test_only]
    use std::option;

    #[test]
    fun test_revenue_hub_creation() {
        genesis::init_for_test();
        
        let test_addr = @0x42;
        
        // Initially no revenue hub exists
        assert!(!revenue_hub_exists(test_addr), 1);
        assert!(get_revenue_balance<RGas>(test_addr) == 0, 2);
        
        // Create revenue hub
        let hub_obj = borrow_or_create_revenue_hub(test_addr);
        let _hub = object::borrow(hub_obj);
        
        // Now revenue hub exists
        assert!(revenue_hub_exists(test_addr), 3);
        assert!(get_revenue_balance<RGas>(test_addr) == 0, 4);
    }

    #[test]
    fun test_revenue_deposit_and_withdrawal() {
        genesis::init_for_test();
        
        let test_addr = @0x42;
        let test_account = moveos_std::account::create_signer_for_testing(test_addr);
        
        // Create some test coins
        let test_coin = gas_coin::mint_for_test(1000u256);
        let generic_coin = coin::convert_coin_to_generic_coin(test_coin);
        
        // Deposit revenue
        let source = create_test_revenue_source(
            std::string::utf8(b"test_source"),
            option::none(),
            std::string::utf8(b"Test deposit")
        );
        deposit_revenue_generic(test_addr, generic_coin, source);
        
        // Check balance
        assert!(get_revenue_balance<RGas>(test_addr) == 1000, 1);
        assert!(get_revenue_by_source(test_addr, std::string::utf8(b"test_source"), type_info::type_name<RGas>()) == 1000, 2);
        
        // Withdraw revenue
        withdraw_revenue<RGas>(&test_account, 500);
        
        // Check remaining balance
        assert!(get_revenue_balance<RGas>(test_addr) == 500, 3);
        
        // Check account coin store received the withdrawal
        assert!(account_coin_store::balance<RGas>(test_addr) == 500, 4);
    }


    #[test]
    #[expected_failure(abort_code = ErrorInsufficientBalance, location = Self)]
    fun test_withdraw_insufficient_balance() {
        genesis::init_for_test();
        
        let test_addr = @0x42;
        let test_account = moveos_std::account::create_signer_for_testing(test_addr);
        
        // Try to withdraw without any revenue
        withdraw_revenue<RGas>(&test_account, 100);
    }

    #[test]
    fun test_preview_withdrawal_fee() {
        genesis::init_for_test();
        
        let test_addr = @0x42;
        
        // Test fee preview (currently returns zero fee)
        let (gross, fee, net) = preview_withdrawal_fee<RGas>(test_addr, 1000);
        assert!(gross == 1000, 1);
        assert!(fee == 0, 2);
        assert!(net == 1000, 3);
    }
}
