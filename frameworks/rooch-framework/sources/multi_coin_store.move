// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::multi_coin_store {

    use std::string;
    use moveos_std::object::ObjectID;
    use moveos_std::object::{Self, Object};
    
    use moveos_std::event;
    use rooch_framework::coin::{Self, GenericCoin, coin_type};

    friend rooch_framework::account_coin_store;

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

    // Data structures

    /// The Balance resource that stores the balance of a specific coin type.
    struct Balance has store {
        value: u256,
    }

    // /// A holder of a specific coin types.
    // /// These are kept in a single resource to ensure locality of data.
    // struct CoinStore<phantom CoinType: key> has key {
    //     balance: Balance,
    //     frozen: bool,
    // }

    /// A holder of a specific coin types.
    /// The non-generic coin store that holds coins by coin_info_id
    struct CoinStoreField has key {
        // /// The coin info ID this store is for
        // coin_info_id: ObjectID,
        // /// Coin type name for easy reference
        /// Coin type name, key
        coin_type: string::String,
        /// Balance of coin this store has
        balance: Balance,
        /// Whether the store is frozen
        frozen: bool,
    }

    // /// The non-generic account coin store that holds all coins for every account
    /// The non-generic coin store that holds all coins for every account
    struct MultiCoinStore has key {}

    // /// Event emitted when a account coin store is created.
    // struct CreateMultiCoinStoreEvent has drop, store, copy {
    //     /// The id of the account coin store that was created
    //     coin_store_id: ObjectID,
    //     /// The account that was created
    //     account: address,
    // }

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
        // /// The type of the coin that was removed
        // coin_type: string::String,
    }

    // //
    // // Public functions
    // //
    //
    // /// Create a new CoinStore Object for `CoinType` and return the Object
    // /// Anyone can create a CoinStore Object for public Coin<CoinType>, the `CoinType` must has `key` and `store` ability
    // public fun create_coin_store<CoinType: key + store>(): Object<CoinStore<CoinType>> {
    //     create_coin_store_internal<CoinType>()
    // }
    //
    // #[private_generics(CoinType)]
    // /// This function is for the `CoinType` module to extend
    // public fun create_coin_store_extend<CoinType: key>(): Object<CoinStore<CoinType>> {
    //     create_coin_store_internal<CoinType>()
    // }
    //
    // /// Remove the CoinStore Object, return the Coin<T> in balance
    // public fun remove_coin_store<CoinType: key>(coin_store_object: Object<CoinStore<CoinType>>): Coin<CoinType> {
    //     let coin_store_id = object::id(&coin_store_object);
    //     let coin_store = object::remove(coin_store_object);
    //
    //     let CoinStore { balance, frozen } = coin_store;
    //     // Cannot remove a frozen CoinStore, because if we allow this, the frozen is meaningless
    //     assert!(!frozen, ErrorCoinStoreIsFrozen);
    //     let Balance { value } = balance;
    //     let coin = coin::pack<CoinType>(value);
    //
    //     let coin_type = type_info::type_name<CoinType>();
    //     event::emit(RemoveEvent {
    //         coin_store_id,
    //         coin_type,
    //     });
    //
    //     coin
    // }
    //
    // public fun balance<CoinType: key>(coin_store_obj: &Object<CoinStore<CoinType>>): u256 {
    //     object::borrow(coin_store_obj).balance.value
    // }
    //
    // public fun is_frozen<CoinType: key>(coin_store_obj: &Object<CoinStore<CoinType>>): bool {
    //     object::borrow(coin_store_obj).frozen
    // }
    //
    // /// Withdraw `amount` Coin<CoinType> from the balance of the passed-in `coin_store`
    // /// This function requires the `CoinType` must has `key` and `store` ability
    // public fun withdraw<CoinType: key + store>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     amount: u256
    // ): Coin<CoinType> {
    //     withdraw_internal(coin_store_obj, amount)
    // }
    //
    // #[private_generics(CoinType)]
    // /// Withdraw `amount` Coin<CoinType> from the balance of the passed-in `coin_store`
    // /// This function is for the `CoinType` module to extend
    // public fun withdraw_extend<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     amount: u256
    // ): Coin<CoinType> {
    //     withdraw_internal(coin_store_obj, amount)
    // }
    //
    // /// Deposit `amount` Coin<CoinType> to the balance of the passed-in `coin_store`
    // /// This function requires the `CoinType` must has `key` and `store` ability
    // public fun deposit<CoinType: key + store>(coin_store_obj: &mut Object<CoinStore<CoinType>>, coin: Coin<CoinType>) {
    //     deposit_internal(coin_store_obj, coin)
    // }
    //
    // #[private_generics(CoinType)]
    // /// Deposit `amount` Coin<CoinType> to the balance of the passed-in `coin_store`
    // /// This function is for the `CoinType` module to extend
    // public fun deposit_extend<CoinType: key>(coin_store_obj: &mut Object<CoinStore<CoinType>>, coin: Coin<CoinType>) {
    //     deposit_internal(coin_store_obj, coin)
    // }
    //
    // // We do not allow to transfer a CoinStore to another account, this function will abort directly.
    // // Because we need to ensure one Account only has one CoinStore for one CoinType
    // // If you want tranfer a CoinStore to another account, you can call `coin_store::remove(Object<CoinStore<CoinType>>)` and deposit the Coin<CoinType> to another account.
    // public fun transfer<CoinType: key>(_coin_store_obj: Object<CoinStore<CoinType>>, _owner: address) {
    //     abort ErrorCoinStoreTransferNotSupported
    // }
    //
    // #[private_generics(CoinType)]
    // /// Borrow a mut CoinStore Object by the coin store id
    // /// This function is for the `CoinType` module to extend
    // public fun borrow_mut_coin_store_extend<CoinType: key>(
    //     object_id: ObjectID
    // ): &mut Object<CoinStore<CoinType>> {
    //     borrow_mut_coin_store_internal(object_id)
    // }
    //
    // #[private_generics(CoinType)]
    // /// Freeze or Unfreeze a CoinStore to prevent withdraw and desposit
    // /// This function is for he `CoinType` module to extend,
    // /// Only the `CoinType` module can freeze or unfreeze a CoinStore by the coin store id
    // public fun freeze_coin_store_extend<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     frozen: bool,
    // ) {
    //     let coin_store_id = object::id(coin_store_obj);
    //     let coin_store = object::borrow_mut(coin_store_obj);
    //     coin_store.frozen = frozen;
    //     let coin_type = type_info::type_name<CoinType>();
    //     event::emit(FreezeEvent {
    //         coin_store_id,
    //         coin_type,
    //         frozen,
    //     });
    // }
    //
    // // Internal functions
    //
    // public(friend) fun create_coin_store_internal<CoinType: key>(): Object<CoinStore<CoinType>> {
    //     coin::check_coin_info_registered<CoinType>();
    //     let coin_type = type_info::type_name<CoinType>();
    //     let coin_store_obj = object::new(CoinStore<CoinType> {
    //         balance: Balance { value: 0 },
    //         frozen: false,
    //     });
    //     event::emit(CreateEvent {
    //         coin_store_id: object::id(&coin_store_obj),
    //         coin_type,
    //     });
    //     coin_store_obj
    // }
    //
    // public(friend) fun create_account_coin_store<CoinType: key>(account: address): ObjectID {
    //     coin::check_coin_info_registered<CoinType>();
    //     let coin_type = type_info::type_name<CoinType>();
    //     let coin_store_obj = object::new_account_named_object(account, CoinStore<CoinType> {
    //         balance: Balance { value: 0 },
    //         frozen: false,
    //     });
    //     let coin_store_id = object::id(&coin_store_obj);
    //     object::transfer_extend(coin_store_obj, account);
    //     event::emit(CreateEvent {
    //         coin_store_id,
    //         coin_type,
    //     });
    //     coin_store_id
    // }
    //
    // public(friend) fun borrow_mut_coin_store_internal<CoinType: key>(
    //     object_id: ObjectID
    // ): &mut Object<CoinStore<CoinType>> {
    //     assert!(object::exists_object_with_type<CoinStore<CoinType>>(object_id), ErrorCoinStoreNotFound);
    //     object::borrow_mut_object_extend<CoinStore<CoinType>>(object_id)
    // }
    //
    // fun check_coin_store_not_frozen<CoinType: key>(coin_store: &CoinStore<CoinType>) {
    //     assert!(!coin_store.frozen, ErrorCoinStoreIsFrozen);
    // }
    //
    // /// Extracts `amount` Coin from the balance of the passed-in `coin_store`
    // fun extract_from_balance<CoinType: key>(coin_store: &mut CoinStore<CoinType>, amount: u256): Coin<CoinType> {
    //     assert!(coin_store.balance.value >= amount, ErrorInsufficientBalance);
    //     coin_store.balance.value = coin_store.balance.value - amount;
    //     coin::pack<CoinType>(amount)
    // }
    //
    // /// "Merges" the given coins to the balance of the passed-in `coin_store`.
    // fun merge_to_balance<CoinType: key>(coin_store: &mut CoinStore<CoinType>, source_coin: Coin<CoinType>) {
    //     let value = coin::unpack(source_coin);
    //     coin_store.balance.value = coin_store.balance.value + value;
    // }
    //
    // public(friend) fun withdraw_internal<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     amount: u256
    // ): Coin<CoinType> {
    //     let object_id = object::id(coin_store_obj);
    //     let coin_store = object::borrow_mut(coin_store_obj);
    //     check_coin_store_not_frozen(coin_store);
    //     let coin = extract_from_balance<CoinType>(coin_store, amount);
    //     let coin_type = type_info::type_name<CoinType>();
    //     event::emit(WithdrawEvent {
    //         coin_store_id: object_id,
    //         coin_type,
    //         amount,
    //     });
    //     coin
    // }
    //
    // /// Deposit `amount` Coin<CoinType> to the balance of the passed-in `coin_store`
    // public(friend) fun deposit_internal<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     coin: Coin<CoinType>
    // ) {
    //     let object_id = object::id(coin_store_obj);
    //     let coin_store = object::borrow_mut(coin_store_obj);
    //     check_coin_store_not_frozen(coin_store);
    //     let amount = coin::value(&coin);
    //     merge_to_balance<CoinType>(coin_store, coin);
    //     let coin_type = type_info::type_name<CoinType>();
    //     event::emit(DepositEvent {
    //         coin_store_id: object_id,
    //         coin_type,
    //         amount,
    //     });
    // }

    // === Non-generic functions ===
    // ====== coin store v2 ======

    // Create a new non-generic coin store
    // /// Create a new CoinStore Object for `CoinType` and return the Object
    // /// Anyone can create a CoinStore Object for public Coin<CoinType>, the `CoinType` must has `key` and `store` ability
    // public fun create_coin_store<CoinType: key + store>(): Object<CoinStore<CoinType>> {
    //     create_coin_store_internal<CoinType>()
    // }
    // public fun create_coin_store(coin_type: string::String): Object<MultiCoinStore> {

    /// Create a new CoinStore Object for `CoinType` and return the Object v2
    /// Anyone can create a CoinStore Object for public Coin<CoinType>, the `CoinType` must has `key` and `store` ability
    // public fun create_coin_store<CoinType: key + store>(): Object<MultiCoinStore> {
    // public fun create_coin_store(coin_type: string::String): Object<MultiCoinStore> {
    //     create_coin_store_internal(coin_type)
    // }

    // #[private_generics(CoinType)]
    // /// This function is for the `CoinType` module to extend v2
    // public fun create_coin_store_extend<CoinType: key>(): Object<MultiCoinStore> {
    //     create_coin_store_internal<CoinType>()
    // }

    public fun exist_coin_store_field(coin_store_obj: &Object<MultiCoinStore>, coin_type: string::String): bool {
        // let account_coin_store_id = account_coin_store_id<CoinType>(addr);
        // object::exists_object_with_type<CoinStore<CoinType>>(account_coin_store_id)

        object::contains_field(coin_store_obj, coin_type)
    }

    // /// Remove the CoinStore Object, return the Coin<T> in balance
    // public fun remove_coin_store<CoinType: key>(coin_store_object: Object<CoinStore<CoinType>>): Coin<CoinType> {
    public fun remove_coin_store_field(coin_store_object: &mut Object<MultiCoinStore>, coin_type: string::String): GenericCoin {
        let coin_store_id = object::id(coin_store_object);
        // let coin_store = object::remove(coin_store_object);
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
        // object::borrow(coin_store_obj).balance.value
    }

    public fun is_frozen(coin_store_obj: &Object<MultiCoinStore>, coin_type: string::String): bool {
        if(object::contains_field(coin_store_obj, coin_type)){
            let coin_store_field = object::borrow_field<MultiCoinStore, string::String, CoinStoreField>(coin_store_obj, coin_type);
            coin_store_field.frozen
        } else {
            false
        }
        // object::borrow(coin_store_obj).frozen
    }

    public fun withdraw(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin_type: string::String,
        amount: u256
    ): GenericCoin {
        withdraw_internal(coin_store_obj, coin_type, amount)
    }

    // #[private_generics(CoinType)]
    // /// Withdraw `amount` Coin<CoinType> from the balance of the passed-in `coin_store`
    // /// This function is for the `CoinType` module to extend
    // public fun withdraw_extend<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     amount: u256
    // )
    // public fun withdraw_extend(
    //     coin_store_obj: &mut Object<MultiCoinStore>,
    //     amount: u256
    // ): GenericCoin {
    //     withdraw_internal(coin_store_obj, amount)
    // }

    public fun deposit(coin_store_obj: &mut Object<MultiCoinStore>, coin: GenericCoin) {
        deposit_internal(coin_store_obj, coin)
    }
    //
    // public fun deposit_extend(coin_store_obj: &mut Object<MultiCoinStore>, coin: GenericCoin) {
    //     deposit_internal(coin_store_obj, coin)
    // }

    public fun transfer(_coin_store_obj: Object<MultiCoinStore>, _owner: address) {
        abort ErrorCoinStoreTransferNotSupported
    }

    // public fun borrow_mut_coin_store_extend(
    //     object_id: ObjectID
    // ): &mut Object<MultiCoinStore> {
    //     borrow_mut_coin_store_internal(object_id)
    // }

    // public fun freeze_coin_store_extend(
    //     coin_store_obj: &mut Object<MultiCoinStore>,
    //     frozen: bool,
    // ) {
    //     let coin_store_id = object::id(coin_store_obj);
    //     let coin_store = object::borrow_mut(coin_store_obj);
    //     coin_store.frozen = frozen;
    //     event::emit(FreezeEvent {
    //         coin_store_id,
    //         coin_type: coin_store.coin_type,
    //         frozen,
    //     });
    // }
    //
    // public(friend) fun create_coin_store_internal(coin_type: string::String): Object<MultiCoinStore> {
    //     coin::check_coin_info_registered(coin_type);
    //     let coin_info_id = coin::coin_info_id(coin_type);
    //     let coin_store_obj = object::new(MultiCoinStore {
    //         coin_info_id,
    //         coin_type,
    //         balance: Balance { value: 0 },
    //         frozen: false,
    //     });
    //     event::emit(CreateEvent {
    //         coin_store_id: object::id(&coin_store_obj),
    //         coin_type,
    //     });
    //     coin_store_obj
    // }

    // public(friend) fun create_account_coin_store(account: address, coin_info_id: ObjectID, coin_type: string::String): ObjectID {
    //     coin::check_coin_info_registered(coin_type);
    //     let coin_store_obj = object::new_account_named_object(account, MultiCoinStore {
    //         coin_info_id,
    //         coin_type,
    //         balance: Balance { value: 0 },
    //         frozen: false,
    //     });
    //     let coin_store_id = object::id(&coin_store_obj);
    //     object::transfer_extend(coin_store_obj, account);
    //     event::emit(CreateEvent {
    //         coin_store_id,
    //         coin_type,
    //     });
    //     coin_store_id
    // }



    // // Direct transfer between coin stores
    // public fun direct_transfer(
    //     from_store_id: ObjectID,
    //     to_store_id: ObjectID,
    //     amount: u256,
    // ) {
    //     let from_store = borrow_mut_coin_store(from_store_id);
    //     let to_store = borrow_mut_coin_store(to_store_id);
    //
    //     // Ensure stores are for the same coin type
    //     assert!(from_store.coin_info_id == to_store.coin_info_id, ErrorCoinTypeMismatch);
    //
    //     // Check if stores are frozen
    //     assert!(!from_store.frozen, ErrorCoinStoreIsFrozen);
    //     assert!(!to_store.frozen, ErrorCoinStoreIsFrozen);
    //
    //     // Check balance
    //     assert!(from_store.value >= amount, ErrorInsufficientBalance);
    //
    //     // Perform transfer
    //     from_store.value = from_store.value - amount;
    //     to_store.value = to_store.value + amount;
    //
    //     // Emit events
    //     event::emit(WithdrawEventV2 {
    //         coin_store_id: from_store_id,
    //         coin_type: from_store.coin_type,
    //         amount,
    //     });
    //
    //     event::emit(DepositEventV2 {
    //         coin_store_id: to_store_id,
    //         coin_type: to_store.coin_type,
    //         amount,
    //     });
    // }

    // Internal v2 functions

    // Create account coin store
    // public(friend) fun create_account_coin_store(account: address, coin_type: string::String): ObjectID {
    public(friend) fun create_multi_coin_store(account: address): ObjectID {
        let coin_store_obj = object::new_account_named_object(
            account,
            MultiCoinStore {}
        );

        let coin_store_id = object::id(&coin_store_obj);
        object::transfer_extend(coin_store_obj, account);

        event::emit(CreateEvent {
            coin_store_id,
            // coin_type
        });

        coin_store_id
    }

    // // public(friend) fun create_coin_store_internal(coin_type: string::String): Object<MultiCoinStore> {
    // // public(friend) fun create_coin_store_internal<CoinType: key>(): Object<MultiCoinStore> {
    // public(friend) fun create_coin_store_internal(coin_type: string::String): Object<MultiCoinStore> {
    //     coin::check_coin_info_registered_by_type_name(coin_type);
    //     // let coin_info_id = coin::coin_info_id<CoinType>();
    //     // let coin_type = type_info::type_name<CoinType>();
    //     let coin_store_obj = object::new(MultiCoinStore {
    //         // coin_info_id,
    //         coin_type,
    //         balance: Balance { value: 0 },
    //         frozen: false,
    //     });
    //     event::emit(CreateEvent {
    //         coin_store_id: object::id(&coin_store_obj),
    //         coin_type,
    //     });
    //     coin_store_obj
    // }

    //
    // // Create account coin store
    // // public(friend) fun create_account_coin_store(account: address, coin_type: string::String): ObjectID {
    // public(friend) fun create_account_coin_store(account: address, coin_type: string::String): ObjectID {
    // // public(friend) fun create_account_coin_store<CoinType: key>(account: address): ObjectID {
    // //     coin::check_coin_info_registered_by_type_name(coin_type);
    //     // let coin_type = type_info::type_name<CoinType>();
    //     // let coin_store_obj = object::new_account_named_object(
    //     //     account,
    //     //     MultiCoinStore {
    //     //         coin_type,
    //     //         balance: Balance { value: 0 },
    //     //         frozen: false,
    //     //     },
    //     //     coin_type
    //     // );
    //
    //     let coin_store_id = create_account_coin_store(account);
    //     let coin_store_obj = object::borrow_mut_object_extend<MultiCoinStore>(coin_store_id);
    //
    //     let coin_store_obj = create_coin_store_internal(coin_type);
    //     let coin_store_id = object::id(&coin_store_obj);
    //     object::transfer_extend(coin_store_obj, account);
    //
    //     object::add_field(coin_store_obj, coin_type, coin_store_id);
    //     coin_store_id
    //
    //     // let coin_store_obj = object::new_account_named_object(
    //     //     account,
    //     //     MultiCoinStore {
    //     //         coin_type,
    //     //         balance: Balance { value: 0 },
    //     //         frozen: false,
    //     //     },
    //     //     coin_type
    //     // );
    //     //
    //     // let coin_store_id = object::id(&coin_store_obj);
    //     // object::transfer_extend(coin_store_obj, account);
    //     //
    //     // event::emit(CreateEvent {
    //     //     coin_store_id,
    //     //     coin_type,
    //     // });
    //     //
    //     // coin_store_id
    // }

    public(friend) fun borrow_mut_coin_store_internal(
        object_id: ObjectID
    ): &mut Object<MultiCoinStore> {
        assert!(object::exists_object_with_type<MultiCoinStore>(object_id), ErrorCoinStoreNotFound);
        object::borrow_mut_object_extend<MultiCoinStore>(object_id)
    }

    // public(friend) fun create_coin_store_field(coin_store_obj: &mut Object<MultiCoinStore>, coin_type: string::String) {
    //     let coin_store_field = CoinStoreField {
    //         coin_type,
    //         balance: Balance { value: 0},
    //         frozen: false,
    //     };
    //     object::add_field(coin_store_obj, coin_type, coin_store_field);
    //     // object::borrow_field<MultiCoinStore, string::String, CoinStoreField>(coin_store, coin_type);
    // }

    public(friend) fun create_coin_store_field_if_not_exist(coin_store_obj: &mut Object<MultiCoinStore>, coin_type: string::String) {
        if(!object::contains_field(coin_store_obj, coin_type)){
            let coin_store_field = CoinStoreField {
                coin_type,
                balance: Balance { value: 0},
                frozen: false,
            };
            object::add_field(coin_store_obj, coin_type, coin_store_field);

            // create_coin_store_field(coin_store_obj, coin_type)
        };
        // object::borrow_field<MultiCoinStore, string::String, CoinStoreField>(coin_store, coin_type);
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
        let value = coin::unpack_generic_coin(source_coin);
        coin_store_field.balance.value = coin_store_field.balance.value + value;
    }

    // public(friend) fun withdraw_internal<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     amount: u256
    // ): Coin<CoinType>
    public(friend) fun withdraw_internal(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin_type: string::String,
        amount: u256
    ): GenericCoin {
        let object_id = object::id(coin_store_obj);
        // let coin_store = object::borrow_mut(coin_store_obj);
        check_coin_store_not_frozen(coin_store_obj, coin_type);
        // let coin_type = coin_store.coin_type;
        let coin = extract_from_balance(coin_store_obj, coin_type, amount);
        event::emit(WithdrawEvent {
            coin_store_id: object_id,
            coin_type,
            amount,
        });
        coin
    }

    // public(friend) fun deposit_internal<CoinType: key>(
    //     coin_store_obj: &mut Object<CoinStore<CoinType>>,
    //     coin: Coin<CoinType>
    // )
    public(friend) fun deposit_internal(
        coin_store_obj: &mut Object<MultiCoinStore>,
        coin: GenericCoin
    ) {
        let object_id = object::id(coin_store_obj);
        let coin_type = coin::coin_type(&coin);
        // let coin_store = object::borrow_mut(coin_store_obj);
        check_coin_store_not_frozen(coin_store_obj, coin_type);
        // let coin_type = coin_store.coin_type;
        let amount = coin::generic_coin_value(&coin);
        merge_to_balance(coin_store_obj, coin);
        event::emit(DepositEvent {
            coin_store_id: object_id,
            coin_type,
            amount,
        });
    }
}