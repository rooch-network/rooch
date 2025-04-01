module rooch_framework::coin_migration {
    use std::string::String;
    use moveos_std::object::{Self, ObjectID, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo, CoinMetadata};
    use rooch_framework::coin_store;
    use rooch_framework::coin_store_v2;

    /// Migrate a generic coin store to a non-generic coin store
    public fun migrate_coin_store<CoinType: key>(
        coin_store_obj: &Object<coin_store::CoinStore<CoinType>>
    ): ObjectID {
        // Get current balance
        let balance = coin_store::balance(coin_store_obj);
        let frozen = coin_store::is_frozen(coin_store_obj);

        // Get coin info
        let coin_type_str = type_info::type_name<CoinType>();
        let coin_info_id = coin::coin_info_id<CoinType>();

        // Create new coin store
        let owner = object::owner(coin_store_obj);
        let new_store_id = coin_store_v2::create_account_coin_store(
            owner,
            coin_info_id,
            coin_type_str
        );

        // If balance is non-zero, we need to transfer it
        if (balance > 0) {
            // Set initial balance for the new store
            let new_store = coin_store_v2::borrow_mut_coin_store(new_store_id);
            new_store.value = balance;

            // Set frozen state
            if (frozen) {
                coin_store_v2::freeze_coin_store(new_store_id, true);
            }
        }

        new_store_id
    }

    /// System migration function for all users' coins
    public entry fun migrate_all_coins(admin: &signer) {
        // Only callable by system admin
        assert!(signer::address_of(admin) == @rooch_framework, ErrorNotAuthorized);

        // Get list of all users with coin stores
        let users = get_all_users_with_coin_stores();

        // For each user
        while (!vector::is_empty(&users)) {
            let user = vector::pop_back(&mut users);
            migrate_user_coins(user);
        }
    }

    /// Migrate all coins for a specific user
    public fun migrate_user_coins(user: address) {
        // Get all coin stores for user
        let coin_stores = get_user_coin_stores(user);

        // For each coin store
        while (!vector::is_empty(&coin_stores)) {
            let (coin_type, store_id) = vector::pop_back(&mut coin_stores);

            // Skip if already migrated
            if (coin_store_v2::is_migrated(store_id)) {
                continue;
            }

            // Perform migration based on coin type
            if (coin_type == "0x3::gas_coin::RGas") {
                let store = object::borrow_object<coin_store::CoinStore<RGas>>(store_id);
                migrate_coin_store<RGas>(store);
            } else if (coin_type == "0x3::usdt::USDT") {
                let store = object::borrow_object<coin_store::CoinStore<USDT>>(store_id);
                migrate_coin_store<USDT>(store);
            }
            // Add more coin types as needed
        }
    }

    /// User-initiated migration for their own coins
    public entry fun migrate_my_coins(user: &signer) {
        migrate_user_coins(signer::address_of(user));
    }

    // Additional logic for frozen stores
    public fun migrate_frozen_coin_store<CoinType: key>(
        coin_store_obj: &Object<coin_store::CoinStore<CoinType>>
    ): ObjectID {
        let new_store_id = migrate_coin_store<CoinType>(coin_store_obj);

        // If original store was frozen, freeze the new one
        if (coin_store::is_frozen(coin_store_obj)) {
            let owner = object::owner(coin_store_obj);
            coin_store_v2::freeze_coin_store(new_store_id, true);

            // Record freeze action in event
            event::emit(MigrationFreezeEvent {
                user: owner,
                coin_type: type_info::type_name<CoinType>(),
                coin_store_id: new_store_id,
            });
        }

        new_store_id
    }

    // Verify migration was successful
    public fun verify_migration<CoinType: key>(
        user: address
    ): bool {
        let coin_type_str = type_info::type_name<CoinType>();

        // Get old store balance
        let old_balance = 0;
        let old_store_id = account_coin_store::account_coin_store_id<CoinType>(user);
        if (object::exists_object_with_type<coin_store::CoinStore<CoinType>>(old_store_id)) {
            let old_store = object::borrow_object<coin_store::CoinStore<CoinType>>(old_store_id);
            old_balance = coin_store::balance(old_store);
        };

        // Get new store balance
        let coin_info_id = coin::coin_info_id<CoinType>();
        let new_store_id = account_coin_store_v2::coin_store_id(user, coin_info_id);
        let new_balance = 0;
        if (object::exists_object_with_type<coin_store_v2::CoinStoreV2>(new_store_id)) {
            let new_store = object::borrow_object<coin_store_v2::CoinStoreV2>(new_store_id);
            new_balance = coin_store_v2::balance(new_store);
        };

        // Verify balances match
        old_balance == new_balance
    }
}