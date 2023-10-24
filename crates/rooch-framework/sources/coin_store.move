// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::coin_store {

    use std::string;
    use std::error;
    use moveos_std::object::{ObjectID};
    use moveos_std::context::{Self, Context};
    use moveos_std::type_info;
    use moveos_std::object_ref::{ObjectRef};
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
    struct CoinStore has key,store {
        coin_type: string::String,
        balance: Balance,
        frozen: bool,
    }

    //
    // Public functions
    //

    /// Create a new CoinStore Object for `CoinType` and return the ObjectRef
    /// Anyone can create a CoinStore Object for public Coin<CoinType>, the `CoinType` must has `key` and `store` ability
    public fun create_coin_store<CoinType: key + store>(ctx: &mut Context): ObjectRef<CoinStore>{
        create_coin_store_internal<CoinType>(ctx) 
    }

    #[private_generics(CoinType)]
    /// This function is for the `CoinType` module to extend
    public fun create_coin_store_extend<CoinType: key>(ctx: &mut Context): ObjectRef<CoinStore> {
        create_coin_store_internal<CoinType>(ctx)
    }
    

     /// Drop the CoinStore, return the Coin<T> in balance 
    public fun drop_coin_store<CoinType: key>(coin_store: CoinStore) : Coin<CoinType> {
        let coin_type = type_info::type_name<CoinType>();
        assert!(coin_store.coin_type == coin_type, error::invalid_argument(ErrorCoinTypeAndStoreMismatch));
        let CoinStore{coin_type:_, balance, frozen:_} = coin_store;
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

    #[private_generics(CoinType)]
    /// Freeze or Unfreeze a CoinStore to prevent withdraw and desposit
    /// This function is for he `CoinType` module to extend,
    /// Only the `CoinType` module can freeze or unfreeze a CoinStore by the coin store id
    public fun freeze_coin_store_extend<CoinType: key>(
        _ctx: &mut Context,
        _coin_store_id: ObjectID,
        _frozen: bool,
    ) {
        //TODO how to provide freeze coin store via coin store id
        // assert!(context::exist_object(ctx, coin_store_id), error::invalid_argument(ErrorCoinStoreNotFound));
        // let coin_store_object = context::borrow_object_mut<CoinStore>(ctx, coin_store_id);
        // object::borrow_mut(coin_store_object).frozen = frozen;
    }

    // Internal functions

    public(friend) fun create_coin_store_internal<CoinType: key>(ctx: &mut Context): ObjectRef<CoinStore>{
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