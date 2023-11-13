// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::coin_store {

    use std::string;
    use std::error;
    use moveos_std::object::{ObjectID};
    use moveos_std::context::{Self, Context};
    use moveos_std::type_info;
    use moveos_std::object::{Self, Object};
    use moveos_std::event;
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

     /// Event emitted when some amount of a coin is deposited into an account.
    struct DepositEvent has drop, store {
        /// The id of the coin store that was deposited to
        coin_store_id: ObjectID,
        /// The type of the coin that was sent
        coin_type: string::String,
        amount: u256,
    }

    /// Event emitted when some amount of a coin is withdrawn from an account.
    struct WithdrawEvent has drop, store {
        /// The id of the coin store that was withdrawn from
        coin_store_id: ObjectID,
        /// The type of the coin that was sent
        coin_type: string::String,
        amount: u256,
    }


    /// The Balance resource that stores the balance of a specific coin type.
    struct Balance has store {
        value: u256,
    }

    /// A holder of a specific coin types.
    /// These are kept in a single resource to ensure locality of data.
    struct CoinStore<phantom CoinType: key> has key {
        coin_type: string::String,
        balance: Balance,
        frozen: bool,
    }

    //
    // Public functions
    //

    /// Create a new CoinStore Object for `CoinType` and return the Object
    /// Anyone can create a CoinStore Object for public Coin<CoinType>, the `CoinType` must has `key` and `store` ability
    public fun create_coin_store<CoinType: key + store>(ctx: &mut Context): Object<CoinStore<CoinType>>{
        create_coin_store_internal<CoinType>(ctx) 
    }

    #[private_generics(CoinType)]
    /// This function is for the `CoinType` module to extend
    public fun create_coin_store_extend<CoinType: key>(ctx: &mut Context): Object<CoinStore<CoinType>> {
        create_coin_store_internal<CoinType>(ctx)
    }
    
    /// Remove the CoinStore Object, return the Coin<T> in balance 
    public fun remove_coin_store<CoinType: key>(coin_store_object: Object<CoinStore<CoinType>>) : Coin<CoinType> {
        let coin_store = object::remove(coin_store_object);
        
        let CoinStore{coin_type:_, balance, frozen} = coin_store;
        // Cannot remove a frozen CoinStore, because if we allow this, the frozen is meaningless
        assert!(!frozen, error::permission_denied(ErrorCoinStoreIsFrozen));
        let Balance{value} = balance;
        coin::pack<CoinType>(value)
    }

    public fun coin_type<CoinType: key>(coin_store_obj: &Object<CoinStore<CoinType>>): string::String {
        object::borrow(coin_store_obj).coin_type
    }

    public fun balance<CoinType: key>(coin_store_obj: &Object<CoinStore<CoinType>>): u256 {
        object::borrow(coin_store_obj).balance.value
    }

    public fun is_frozen<CoinType: key>(coin_store_obj: &Object<CoinStore<CoinType>>): bool {
        object::borrow(coin_store_obj).frozen
    }

    /// Withdraw `amount` Coin<CoinType> from the balance of the passed-in `coin_store`
    public fun withdraw<CoinType: key>(coin_store_obj: &mut Object<CoinStore<CoinType>>, amount: u256) : Coin<CoinType> {
        let object_id = object::id(coin_store_obj);
        let coin_store = object::borrow_mut(coin_store_obj);
        check_coin_store_not_frozen(coin_store);
        let coin = extract_from_balance<CoinType>(coin_store, amount);
        event::emit(WithdrawEvent{
            coin_store_id: object_id,
            coin_type: coin_store.coin_type,
            amount: amount,
        });
        coin
    }

    /// Deposit `amount` Coin<CoinType> to the balance of the passed-in `coin_store`
    public fun deposit<CoinType: key>(coin_store_obj: &mut Object<CoinStore<CoinType>>, coin: Coin<CoinType>) {
        let object_id = object::id(coin_store_obj);
        let coin_store = object::borrow_mut(coin_store_obj);
        check_coin_store_not_frozen(coin_store);
        let amount = coin::value(&coin);
        merge_to_balance<CoinType>(coin_store, coin);
        event::emit(DepositEvent{
            coin_store_id: object_id,
            coin_type: coin_store.coin_type,
            amount,
        });
    }

    // We do not allow to transfer a CoinStore to another account, CoinStore is default ownerd by the system.
    // Only provide a internal function for account_coin_store.
    public fun transfer<CoinType: key>(coin_store_obj: Object<CoinStore<CoinType>>, owner: address){
        // Cannot transfer a frozen CoinStore
        // We do not use the frozen Object to represent the frozen CoinStore, because we want allow the T module to unfreeze the CoinStore
        assert!(!object::borrow(&coin_store_obj).frozen, error::permission_denied(ErrorCoinStoreIsFrozen));
        //TODO if the owner has the CoinStore<CoinType>, we should merge this CoinStore<CoinType> to the owner's CoinStore<CoinType>
        //So, we need to migrate account_coin_store to coin_store module.
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
        assert!(context::exists_object<CoinStore<CoinType>>(ctx, coin_store_id), error::invalid_argument(ErrorCoinStoreNotFound));
        let coin_store_object = context::borrow_mut_object_extend<CoinStore<CoinType>>(ctx, coin_store_id);
        object::borrow_mut(coin_store_object).frozen = frozen;
    }

    // Internal functions

    public(friend) fun create_coin_store_internal<CoinType: key>(ctx: &mut Context): Object<CoinStore<CoinType>>{
        coin::check_coin_info_registered<CoinType>(ctx);
        context::new_object(ctx, CoinStore<CoinType>{
            coin_type: type_info::type_name<CoinType>(),
            balance: Balance { value: 0 },
            frozen: false,
        })
    }

    public(friend) fun create_account_coin_store<CoinType: key>(ctx: &mut Context, account: address): &mut Object<CoinStore<CoinType>>{
        coin::check_coin_info_registered<CoinType>(ctx);
        context::new_account_singleton(ctx, account, CoinStore<CoinType>{
            coin_type: type_info::type_name<CoinType>(),
            balance: Balance { value: 0 },
            frozen: false,
        })
    }

    public(friend) fun borrow_mut_coin_store<CoinType: key>(ctx: &mut Context, object_id: ObjectID): &mut Object<CoinStore<CoinType>>{
        context::borrow_mut_object_extend<CoinStore<CoinType>>(ctx, object_id)
    }

    fun check_coin_store_not_frozen<CoinType: key>(coin_store: &CoinStore<CoinType>) {
        assert!(!coin_store.frozen,error::permission_denied(ErrorCoinStoreIsFrozen));
    }

    /// Extracts `amount` Coin from the balance of the passed-in `coin_store`
    fun extract_from_balance<CoinType: key>(coin_store: &mut CoinStore<CoinType>, amount: u256): Coin<CoinType> {
        assert!(coin_store.balance.value >= amount, error::invalid_argument(ErrorInSufficientBalance));
        coin_store.balance.value = coin_store.balance.value - amount;
        coin::pack<CoinType>(amount)
    }

    /// "Merges" the given coins to the balance of the passed-in `coin_store`.
    fun merge_to_balance<CoinType: key>(coin_store: &mut CoinStore<CoinType>, source_coin: Coin<CoinType>) {
        let value = coin::unpack(source_coin);
        coin_store.balance.value = coin_store.balance.value + value;
    }
}