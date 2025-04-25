// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::multi_coin_store {

    use std::string;
    use moveos_std::type_info::type_name;
    use moveos_std::ability;
    use moveos_std::object::ObjectID;
    use moveos_std::object::{Self, Object};
    
    use moveos_std::event;
    use rooch_framework::coin::{Self, GenericCoin, Coin};

    friend rooch_framework::account_coin_store;
    friend rooch_framework::coin_migration;

    // Error codes

    /// The CoinStore is not found in the global object store
    const ErrorCoinStoreNotFound: u64 = 1;

    /// CoinStore is frozen. Coins cannot be deposited or withdrawn
    const ErrorCoinStoreIsFrozen: u64 = 2;

    /// The CoinType parameter and CoinType in CoinStore do not match
    const ErrorCoinTypeAndStoreMismatch: u64 = 3;

    /// Not enough balance to withdraw from CoinStore
    const ErrorInsufficientBalance: u64 = 4;

    /// Transfer is not supported for CoinStore
    const ErrorCoinStoreTransferNotSupported: u64 = 5;

    /// Coin type should have key and store ability
    const ErrorCoinTypeShouldHaveKeyAndStoreAbility: u64 = 6;

    /// Coin type should have key ability
    const ErrorCoinTypeShouldHaveKeyAbility: u64 = 7;

    // Data structures

    /// The Balance resource that stores the balance of a specific coin type.
    struct Balance has store {
        value: u256,
    }

    /// A holder of a specific coin types.
    /// The non-generic coin store field that holds coins by coin_type
    struct CoinStoreField has key, store {
        /// Coin type name, key
        coin_type: string::String,
        /// Balance of coin this store has
        balance: Balance,
        /// Whether the store is frozen
        frozen: bool,
    }

    /// The non-generic coin store that holds all coins for every account
    struct MultiCoinStore has key {}

    /// Event emitted when a coin store is created.
    struct CreateEvent has drop, store, copy {
        /// The id of the coin store that was created
        coin_store_id: ObjectID,
        // /// The type of the coin that was created
        // coin_type: string::String,
    }

    /// Event emitted when some amount of a coin is deposited into a coin store.
    struct DepositEvent has drop, store, copy {
        /// The id of the coin store that was deposited to
        coin_store_id: ObjectID,
        /// The type of the coin that was sent
        coin_type: string::String,
        amount: u256,
    }

    /// Event emitted when some amount of a coin is withdrawn from a coin store.
    struct WithdrawEvent has drop, store, copy {
        /// The id of the coin store that was withdrawn from
        coin_store_id: ObjectID,
        /// The type of the coin that was sent
        coin_type: string::String,
        amount: u256,
    }

    /// Event emitted when a coin store is frozen or unfrozen.
    struct FreezeEvent has drop, store, copy {
        /// The id of the coin store that was frozen or unfrozen
        coin_store_id: ObjectID,
        /// The type of the coin that was frozen or unfrozen
        coin_type: string::String,
        frozen: bool,
    }

    /// Event emitted when a coin store is removed.
    struct RemoveEvent has drop, store, copy {
        /// The id of the coin store that was removed
        coin_store_id: ObjectID,
        /// The type of the coin that was removed
        coin_type: string::String,
    }

    //
    // Public functions
    //

    public fun exist_coin_store_field(coin_store_obj: &Object<MultiCoinStore>, coin_type: string::String): bool {
        object::contains_field(coin_store_obj, coin_type)
    }

    // /// Remove the MultiCoinStore field, return the GenericCoin in balance
    public fun remove_coin_store_field(coin_store_object: &mut Object<MultiCoinStore>, coin_type: string::String): GenericCoin {
        ensure_coin_type_has_key_ability(coin_type);
        let coin_store_id = object::id(coin_store_object);
        let coin_store_field = object::remove_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_object, coin_type);

        let CoinStoreField { coin_type, balance, frozen } = coin_store_field;
        // Cannot remove a frozen CoinStore, because if we allow this, the frozen is meaningless
        assert!(!frozen, ErrorCoinStoreIsFrozen);
        let Balance { value } = balance;
        let coin = coin::pack_generic_coin(coin_type, value);

        event::emit(RemoveEvent {
            coin_store_id,
            coin_type,
        });

        coin
    }

    public fun balance(coin_store_obj: &Object<MultiCoinStore>, coin_type: string::String): u256 {
        if(object::contains_field(coin_store_obj, coin_type)){
            let coin_store_field = object::borrow_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
            coin_store_field.balance.value
        } else {
            0
        }
    }

    public fun is_frozen(coin_store_obj: &Object<MultiCoinStore>, coin_type: string::String): bool {
        if(object::contains_field(coin_store_obj, coin_type)){
            let coin_store_field = object::borrow_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
            coin_store_field.frozen
        } else {
            false
        }
    }

    public fun withdraw(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin_type: string::String,
        amount: u256
    ): GenericCoin {
        ensure_coin_type_has_key_and_store_ability(coin_type);
        withdraw_internal(coin_store_obj, coin_type, amount)
    }

    public fun deposit(coin_store_obj: &mut Object<MultiCoinStore>, coin: GenericCoin) {
        let coin_type = coin::coin_type(&coin);
        ensure_coin_type_has_key_and_store_ability(coin_type);
        deposit_internal(coin_store_obj, coin)
    }

    public fun transfer(_coin_store_obj: Object<MultiCoinStore>, _owner: address) {
        abort ErrorCoinStoreTransferNotSupported
    }

    fun ensure_coin_type_has_key_and_store_ability(coin_type: string::String) {
        let coin_type_abilities = ability::native_get_abilities(coin_type);
        assert!(ability::has_key(coin_type_abilities), ErrorCoinTypeShouldHaveKeyAndStoreAbility);
        assert!(ability::has_store(coin_type_abilities), ErrorCoinTypeShouldHaveKeyAndStoreAbility);
    }

    fun ensure_coin_type_has_key_ability(coin_type: string::String) {
        let coin_type_abilities = ability::native_get_abilities(coin_type);
        assert!(ability::has_key(coin_type_abilities), ErrorCoinTypeShouldHaveKeyAbility);
    }

    #[private_generics(CoinType)]
    /// Withdraw `amount` Coin<CoinType> from the balance of the passed-in `multi_coin_store`
    /// This function is for the `CoinType` module to extend
    public fun withdraw_extend<CoinType: key>(
        coin_store_obj: &mut Object<MultiCoinStore>,
        amount: u256
    ): GenericCoin {
        let coin_type = type_name<CoinType>();
        withdraw_internal(coin_store_obj, coin_type, amount)
    }

    #[private_generics(CoinType)]
    /// Deposit `amount` Coin<CoinType> to the balance of the passed-in `multi_coin_store`
    /// This function is for the `CoinType` module to extend
    public fun deposit_extend<CoinType: key>(coin_store_obj: &mut Object<MultiCoinStore>, coin: Coin<CoinType>) {
        let generic_coin = coin::convert_coin_to_generic_coin(coin);
        deposit_internal(coin_store_obj, generic_coin)
    }

    #[private_generics(CoinType)]
    /// Freeze or Unfreeze a MultiCoinStore field to prevent withdraw and desposit
    /// This function is for he `CoinType` module to extend,
    /// Only the `CoinType` module can freeze or unfreeze a MultiCoinStore field by the coin store id
    public fun freeze_coin_store_extend<CoinType: key>(
        coin_store_obj: &mut Object<MultiCoinStore>,
        frozen: bool,
    ) {
        let coin_type = type_name<CoinType>();
        freeze_coin_store_internal(coin_store_obj, coin_type, frozen)
    }

    // Internal functions

    // Create multi coin store
    public(friend) fun create_multi_coin_store(account: address): ObjectID {
        let coin_store_obj = object::new_account_named_object(
            account,
            MultiCoinStore {}
        );

        let coin_store_id = object::id(&coin_store_obj);
        object::transfer_extend(coin_store_obj, account);

        event::emit(CreateEvent {
            coin_store_id,
        });

        coin_store_id
    }

    public(friend) fun borrow_mut_coin_store_internal(
        object_id: ObjectID
    ): &mut Object<MultiCoinStore> {
        assert!(object::exists_object_with_type<MultiCoinStore>(object_id), ErrorCoinStoreNotFound);
        object::borrow_mut_object_extend<MultiCoinStore>(object_id)
    }


    public(friend) fun create_coin_store_field_if_not_exist(coin_store_obj: &mut Object<MultiCoinStore>, coin_type: string::String) {
        if(!object::contains_field(coin_store_obj, coin_type)){
            let coin_store_field = CoinStoreField {
                coin_type,
                balance: Balance { value: 0},
                frozen: false,
            };
            object::add_field(coin_store_obj, coin_type, coin_store_field);
        };
    }
    
    fun check_coin_store_not_frozen(coin_store_obj: &Object<MultiCoinStore>, coin_type: string::String) {
        if(object::contains_field(coin_store_obj, coin_type)){
            let coin_store_field = object::borrow_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
            assert!(!coin_store_field.frozen, ErrorCoinStoreIsFrozen);
        }
    }

    fun extract_from_balance(coin_store_obj: &mut Object<MultiCoinStore>, coin_type: string::String, amount: u256): GenericCoin {
        create_coin_store_field_if_not_exist(coin_store_obj, coin_type);
        let coin_store_field = object::borrow_mut_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
        assert!(coin_store_field.balance.value >= amount, ErrorInsufficientBalance);
        coin_store_field.balance.value = coin_store_field.balance.value - amount;
        coin::pack_generic_coin(coin_type, amount)
    }

    fun merge_to_balance(coin_store_obj: &mut Object<MultiCoinStore>, source_coin: GenericCoin) {
        let coin_type = coin::coin_type(&source_coin);
        create_coin_store_field_if_not_exist(coin_store_obj, coin_type);
        let coin_store_field = object::borrow_mut_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
        let (source_coin_type, value) = coin::unpack_generic_coin(source_coin);
        assert!(coin_type == source_coin_type, ErrorCoinTypeAndStoreMismatch);
        coin_store_field.balance.value = coin_store_field.balance.value + value;
    }

    public(friend) fun withdraw_internal(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin_type: string::String,
        amount: u256
    ): GenericCoin {
        let object_id = object::id(coin_store_obj);
        check_coin_store_not_frozen(coin_store_obj, coin_type);
        let coin = extract_from_balance(coin_store_obj, coin_type, amount);
        event::emit(WithdrawEvent {
            coin_store_id: object_id,
            coin_type,
            amount,
        });
        coin
    }

    public(friend) fun deposit_internal(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin: GenericCoin
    ) {
        let object_id = object::id(coin_store_obj);
        let coin_type = coin::coin_type(&coin);
        check_coin_store_not_frozen(coin_store_obj, coin_type);
        let amount = coin::generic_coin_value(&coin);
        merge_to_balance(coin_store_obj, coin);
        event::emit(DepositEvent {
            coin_store_id: object_id,
            coin_type,
            amount,
        });
    }

    public(friend) fun freeze_coin_store_internal(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin_type: string::String,
        frozen: bool,
    ) {
        let coin_store_id = object::id(coin_store_obj);
        create_coin_store_field_if_not_exist(coin_store_obj, coin_type);
        let coin_store_field = object::borrow_mut_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
        coin_store_field.frozen = frozen;
        event::emit(FreezeEvent {
            coin_store_id,
            coin_type,
            frozen,
        });
    }

    #[test_only]
    public fun create_multi_coin_store_for_test(new_address: address): ObjectID {
        create_multi_coin_store(new_address)
    }

    #[test_only]
    public fun create_coin_store_field_if_not_exist_for_test(coin_store_obj: &mut Object<MultiCoinStore>, coin_type: string::String) {
        create_coin_store_field_if_not_exist(coin_store_obj, coin_type)
    }

    #[test_only]
    public fun freeze_coin_store_for_test(coin_store_obj: &mut Object<MultiCoinStore>, coin_type: string::String, frozen: bool) {
        freeze_coin_store_internal(coin_store_obj, coin_type, frozen)
    }

    #[test_only]
    public fun borrow_mut_coin_store_for_test(
        object_id: ObjectID
    ): &mut Object<MultiCoinStore> {
        borrow_mut_coin_store_internal(object_id)
    }
}