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
        let _hub_obj = borrow_or_create_revenue_hub(sender);
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
        
        // Emit withdrawal event with per-hub handle for account-scoped queries
        let hub_id = object::id(hub_obj);
        let handle_id = revenue_event_handle_id<RevenueWithdrawnEvent>(hub_id);
        event::emit_with_handle(
            handle_id,
            RevenueWithdrawnEvent {
                hub_id,
                owner: owner_addr,
                coin_type,
                amount,
                fee_amount: 0u256,      // No fees currently implemented
                net_amount: amount,     // Net equals gross when no fees
                fee_rate_bps: 0u16,     // No fee rate currently
            },
        );
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
        
        // Emit deposit event with per-hub handle for account-scoped queries
        let hub_id = object::id(hub_obj);
        let handle_id = revenue_event_handle_id<RevenueDepositedEvent>(hub_id);
        event::emit_with_handle(
            handle_id,
            RevenueDepositedEvent {
                hub_id,
                owner: account,
                coin_type,
                amount,
                source_type: source.source_type,
                source_id: source.source_id,
                source_description: source.description,
            },
        );
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
            
            // Emit creation event with per-hub handle for account-scoped queries
            let handle_id = revenue_event_handle_id<RevenueHubCreatedEvent>(hub_obj_id);
            event::emit_with_handle(
                handle_id,
                RevenueHubCreatedEvent {
                    hub_id: hub_obj_id,
                    owner,
                },
            );
        };
        object::borrow_mut_object_extend<PaymentRevenueHub>(hub_obj_id)
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

    /// Derive the per-hub event handle for any revenue event type
    fun revenue_event_handle_id<T>(hub_id: ObjectID): ObjectID {
        event::custom_event_handle_id<ObjectID, T>(hub_id)
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

    // === Test Only Functions ===

    #[test_only]
    /// Test-only function to access borrow_or_create_revenue_hub
    public fun borrow_or_create_revenue_hub_for_test(owner: address): &mut Object<PaymentRevenueHub> {
        borrow_or_create_revenue_hub(owner)
    }

    #[test_only]
    /// Test-only function to access revenue_event_handle_id
    public fun revenue_event_handle_id_for_test<T>(hub_id: ObjectID): ObjectID {
        revenue_event_handle_id<T>(hub_id)
    }

    #[test_only]
    /// Test-only function to deposit revenue (exposes friend function for testing)
    public fun deposit_revenue_generic_for_test(
        account: address,
        coin: GenericCoin,
        source: RevenueSource,
    ) {
        deposit_revenue_generic(account, coin, source);
    }

}
