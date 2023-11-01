// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::coin_store {

    use std::string;
    use std::error;
    use moveos_std::object::{ObjectID};
    use moveos_std::context::{Self, Context};
    use moveos_std::type_info;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin};

    friend rooch_framework::account_coin_store;

    // Error codes

    /// The CoinStore is not found in the global object store
    const ErrorCoinStoreNotFound: u64 = 1;

    /// CoinStore is frozen. Coins cannot be deposited or withdrawn
    const ErrorCoinStoreIsFrozen: u64 = 2;

    /// The CoinType parameter and CoinType in CoinStore do not match
    const ErrorCoinTypeAndStoreMismatch: u64 = 3;

    /// Not enough balance to withdraw from CoinStore
    const ErrorInSufficientBalance: u64 = 4;


    /// The Balance resource that stores the balance of a specific coin type.
    struct Balance has store {
        value: u256,
    }

    /// A holder of a specific coin types.
    /// These are kept in a single resource to ensure locality of data.
    struct CoinStore has key {
        coin_type: string::String,
        balance: Balance,
        frozen: bool,
    }

    //
    // Public functions
    //

    /// Create a new CoinStore Object for `CoinType` and return the Object
    /// Anyone can create a CoinStore Object for public Coin<CoinType>, the `CoinType` must has `key` and `store` ability
    public fun create_coin_store<CoinType: key + store>(ctx: &mut Context): Object<CoinStore>{
        create_coin_store_internal<CoinType>(ctx) 
    }

    #[private_generics(CoinType)]
    /// This function is for the `CoinType` module to extend
    public fun create_coin_store_extend<CoinType: key>(ctx: &mut Context): Object<CoinStore> {
        create_coin_store_internal<CoinType>(ctx)
    }
    
    /// Remove the CoinStore Object, return the Coin<T> in balance 
    public fun remove_coin_store<CoinType: key>(coin_store_object: Object<CoinStore>) : Coin<CoinType> {
        let coin_store = object::remove(coin_store_object);
        let coin_type = type_info::type_name<CoinType>();
        assert!(coin_store.coin_type == coin_type, error::invalid_argument(ErrorCoinTypeAndStoreMismatch));
        let CoinStore{coin_type:_, balance, frozen} = coin_store;
        // Cannot remove a frozen CoinStore, because if we allow this, the frozen is meaningless
        assert!(!frozen, error::permission_denied(ErrorCoinStoreIsFrozen));
        let Balance{value} = balance;
        coin::pack<CoinType>(value)
    }

    public fun coin_type(self: &CoinStore): string::String {
        self.coin_type
    }

    public fun balance(self: &CoinStore): u256 {
        self.balance.value
    }

    public fun is_frozen(self: &CoinStore): bool {
        self.frozen
    }

    /// Withdraw `amount` Coin<CoinType> from the balance of the passed-in `coin_store`
    public fun withdraw<CoinType: key>(coin_store: &mut CoinStore, amount: u256) : Coin<CoinType> {
        check_coin_store_not_frozen(coin_store);
        extract_from_balance<CoinType>(coin_store, amount)
    }

    /// Deposit `amount` Coin<CoinType> to the balance of the passed-in `coin_store`
    public fun deposit<CoinType: key>(coin_store: &mut CoinStore, coin: Coin<CoinType>) {
        check_coin_store_not_frozen(coin_store);
        merge_to_balance<CoinType>(coin_store, coin);
    }

    // We do not allow to transfer a CoinStore to another account, CoinStore is default ownerd by the system.
    // Only provide a internal function for account_coin_store.
    public fun transfer(coin_store_obj: Object<CoinStore>, owner: address){
        // Cannot transfer a frozen CoinStore
        // We do not use the frozen Object to represent the frozen CoinStore, because we want allow the T module to unfreeze the CoinStore
        assert!(!object::borrow(&coin_store_obj).frozen, error::permission_denied(ErrorCoinStoreIsFrozen));
        object::transfer_extend(coin_store_obj, owner)
    }

    #[private_generics(CoinType)]
    /// Freeze or Unfreeze a CoinStore to prevent withdraw and desposit
    /// This function is for he `CoinType` module to extend,
    /// Only the `CoinType` module can freeze or unfreeze a CoinStore by the coin store id
    public fun freeze_coin_store_extend<CoinType: key>(
        ctx: &mut Context,
        coin_store_id: ObjectID,
        frozen: bool,
    ) {
        assert!(context::exist_object<CoinStore>(ctx, coin_store_id), error::invalid_argument(ErrorCoinStoreNotFound));
        let coin_store_object = context::borrow_mut_object_extend<CoinStore>(ctx, coin_store_id);
        object::borrow_mut(coin_store_object).frozen = frozen;
    }

    // Internal functions

    public(friend) fun create_coin_store_internal<CoinType: key>(ctx: &mut Context): Object<CoinStore>{
        coin::check_coin_info_registered<CoinType>(ctx);
        context::new_object(ctx, CoinStore{
            coin_type: type_info::type_name<CoinType>(),
            balance: Balance { value: 0 },
            frozen: false,
        })
    }

    fun check_coin_store_not_frozen(coin_store: &CoinStore) {
        assert!(!coin_store.frozen,error::permission_denied(ErrorCoinStoreIsFrozen));
    }

    /// Extracts `amount` Coin from the balance of the passed-in `coin_store`
    fun extract_from_balance<CoinType: key>(coin_store: &mut CoinStore, amount: u256): Coin<CoinType> {
        let coin_type = type_info::type_name<CoinType>();
        assert!(coin_store.coin_type == coin_type, error::invalid_argument(ErrorCoinTypeAndStoreMismatch));
        assert!(coin_store.balance.value >= amount, error::invalid_argument(ErrorInSufficientBalance));
        coin_store.balance.value = coin_store.balance.value - amount;
        coin::pack<CoinType>(amount)
    }

    /// "Merges" the given coins to the balance of the passed-in `coin_store`.
    fun merge_to_balance<CoinType: key>(coin_store: &mut CoinStore, source_coin: Coin<CoinType>) {
        let coin_type = type_info::type_name<CoinType>();
        assert!(coin_store.coin_type == coin_type, error::invalid_argument(ErrorCoinTypeAndStoreMismatch));
        let value = coin::unpack(source_coin);
        coin_store.balance.value = coin_store.balance.value + value;
    }
}