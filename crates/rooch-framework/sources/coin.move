// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::string;
    use std::error;
    use moveos_std::object::ObjectID;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::account_storage;
    use moveos_std::context::{Context};
    use moveos_std::event;
    use moveos_std::type_info::{Self, type_of};
 
    friend rooch_framework::genesis;
    friend rooch_framework::coin_store;

    //
    // Errors.
    //

    /// `CoinType` is not registered as a coin
    const ErrorCoinInfoNotRegistered: u64 = 1;

    /// `CoinType` is already registered as a coin
    const ErrorCoinInfoAlreadyRegistered: u64 = 2;

    /// Not enough coins to extract
    const ErrorInSufficientBalance: u64 = 3;

    /// Cannot destroy non-zero coins
    const ErrorDestroyOfNonZeroCoin: u64 = 4;

    /// Coin amount cannot be zero
    const ErrorZeroCoinAmount: u64 = 5;

    /// Name of the coin is too long
    const ErrorCoinNameTooLong: u64 = 6;

    /// Symbol of the coin is too long
    const ErrorCoinSymbolTooLong: u64 = 7;

    /// Global CoinInfos should exist
    const ErrorCoinInfosNotFound: u64 = 8;


    //
    // Constants
    //

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    /// Core data structures

    /// Main structure representing a coin.
    /// Note the `CoinType` must have `key` ability.
    /// if the `CoinType` has `store` ability, the `Coin` is a public coin, the user can operate it directly by coin module's function.
    /// Otherwise, the `Coin` is a private coin, the user can only operate it by `CoinType` module's function.
    /// The Coin has no ability, it is a hot potato type, only can handle by Coin module.
    struct Coin<phantom CoinType : key> {
        /// Amount of coin this address has.
        /// Following the ERC20 standard, both asset balance and supply are expressed in u256
        value: u256,
    }

    /// Maximum possible aggregatable coin value.
    const MAX_U64: u128 = 18446744073709551615;

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    const MAX_U256: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

    /// Information about a specific coin type. Stored in the global CoinInfos table.
    struct CoinInfo has store {
        /// Type of the coin: `address::my_module::XCoin`, same as `moveos_std::type_info::type_name<CoinType>()`.
        /// The name and symbol can repeat across different coin types, but the coin type must be unique.
        coin_type: string::String,
        /// Name of the coin.
        name: string::String,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: string::String,
        /// Number of decimals used to get its user representation.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        /// The total value for the coin represented by coin type. Mutable.
        supply: u256,
    }

    /// A resource that holds the CoinInfo for all accounts.
    struct CoinInfos has key {
        coin_infos: Table<string::String, CoinInfo>,
    }

    /// Event emitted when coin minted.
    struct MintEvent has drop, store {
        /// The type of coin that was minted
        coin_type: string::String,
        /// coins added to the system
        amount: u256,
    }

    /// Event emitted when coin burned.
    struct BurnEvent has drop, store {
         /// The type of coin that was burned
        coin_type: string::String,
        /// coins removed from the system
        amount: u256,
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer) {
        let coin_infos = CoinInfos {
            coin_infos: table::new(ctx),
        };
        account_storage::global_move_to(ctx, genesis_account, coin_infos);
    }

    
    //
    // Public functions
    //

    /// A helper function that returns the address of CoinType.
    fun coin_address<CoinType: key>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }

    /// A helper function that check the `CoinType` is registered, if not, abort.
    public fun check_coin_info_registered<CoinType: key>(ctx: &Context){
        assert!(is_registered<CoinType>(ctx), error::not_found(ErrorCoinInfoNotRegistered));
    }

    /// Returns `true` if the type `CoinType` is an registered coin.
    public fun is_registered<CoinType: key>(ctx: &Context): bool {
        if (account_storage::global_exists<CoinInfos>(ctx, @rooch_framework)) {
            let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
            let coin_type = type_info::type_name<CoinType>();
            table::contains(&coin_infos.coin_infos, coin_type)
        } else {
            false
        }
    }

    /// Returns the name of the coin.
    public fun name<CoinType: key>(ctx: &Context): string::String {
        borrow_coin_info<CoinType>(ctx).name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType: key>(ctx: &Context): string::String {
        borrow_coin_info<CoinType>(ctx).symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType: key>(ctx: &Context): u8 {
        borrow_coin_info<CoinType>(ctx).decimals
    }

    /// Returns the amount of coin in existence.
    public fun supply<CoinType: key>(ctx: &Context): u256 {
        borrow_coin_info<CoinType>(ctx).supply
    }

    /// Return true if the type `CoinType1` is same with `CoinType2`
    public fun is_same_coin<CoinType1, CoinType2>(): bool {
        return type_of<CoinType1>() == type_of<CoinType2>()
    }

    /// Return CoinInfos table handle
    public fun coin_infos_handle(ctx: &Context): ObjectID {
        // coin info ensured via the Genesis transaction, so it should always exist
        assert!(account_storage::global_exists<CoinInfos>(ctx, @rooch_framework), error::invalid_argument(ErrorCoinInfosNotFound));
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        *table::handle(&coin_infos.coin_infos)
    }

    
    /// Destroys a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Coin`. 
    public fun destroy_zero<CoinType: key>(zero_coin: Coin<CoinType>) {
        let Coin { value } = zero_coin;
        assert!(value == 0, error::invalid_argument(ErrorDestroyOfNonZeroCoin))
    }

    /// Extracts `amount` from the passed-in `coin`, where the original coin is modified in place.
    public fun extract<CoinType: key>(coin: &mut Coin<CoinType>, amount: u256): Coin<CoinType> {
        assert!(coin.value >= amount, error::invalid_argument(ErrorInSufficientBalance));
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    /// Extracts the entire amount from the passed-in `coin`, where the original coin is modified in place.
    public fun extract_all<CoinType: key>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let total_value = coin.value;
        coin.value = 0;
        Coin { value: total_value }
    }

    /// "Merges" the two given coins.  The coin passed in as `dst_coin` will have a value equal
    /// to the sum of the two coins (`dst_coin` and `source_coin`).
    public fun merge<CoinType: key>(dst_coin: &mut Coin<CoinType>, source_coin: Coin<CoinType>) {
        let Coin { value } = source_coin;
        dst_coin.value = dst_coin.value + value;
    }

    /// Returns the `value` passed in `coin`.
    public fun value<CoinType: key>(coin: &Coin<CoinType>): u256 {
        coin.value
    }

    /// Create a new `Coin<CoinType>` with a value of `0`.
    public fun zero<CoinType: key>(): Coin<CoinType> {
        Coin<CoinType> {
            value: 0
        }
    }

    //
    // Extend functions
    //

    #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType`
    /// This function is protected by `private_generics`, so it can only be called by the `CoinType` module.
    public fun register_extend<CoinType: key>(
        ctx: &mut Context,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ){
        
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        let coin_type = type_info::type_name<CoinType>();
        
        assert!(
            !table::contains(&coin_infos.coin_infos, coin_type),
            error::already_exists(ErrorCoinInfoAlreadyRegistered),
        ); 

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ErrorCoinNameTooLong));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ErrorCoinSymbolTooLong));

        let coin_info = CoinInfo {
            coin_type,
            name,
            symbol,
            decimals,
            supply: 0u256,
        };
        table::add(&mut coin_infos.coin_infos, coin_type, coin_info);
    }

    #[private_generics(CoinType)]
    /// Mint new `Coin`, this function is only called by the `CoinType` module, for the developer to extend custom mint logic
    public fun mint_extend<CoinType: key>(ctx: &mut Context,amount: u256) : Coin<CoinType> {
        mint_internal<CoinType>(ctx, amount)
    }

    #[private_generics(CoinType)]
    /// Burn `coin`
    /// This function is only called by the `CoinType` module, for the developer to extend custom burn logic
    public fun burn_extend<CoinType: key>(
        ctx: &mut Context,
        coin: Coin<CoinType>,
    ) {
        burn_internal(ctx, coin) 
    }

    //
    // Internal functions
    //

    fun mint_internal<CoinType: key>(ctx: &mut Context,
        amount: u256): Coin<CoinType>{
        let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply + amount;
        let coin_type = type_info::type_name<CoinType>();
        event::emit<MintEvent>(ctx, MintEvent {
            coin_type,
            amount,
        });
        Coin<CoinType> { value: amount }
    }

    fun burn_internal<CoinType: key>(
        ctx: &mut Context,
        coin: Coin<CoinType>,
    ) {
        let Coin { value: amount } = coin;

        let coin_type = type_info::type_name<CoinType>();
        let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply - amount;
        event::emit<BurnEvent>(ctx, BurnEvent {
            coin_type,
            amount,
        });
    }

    fun borrow_coin_info<CoinType: key>(ctx: &Context): &CoinInfo {
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        let coin_type = type_info::type_name<CoinType>();
        check_coin_info_registered_internal(coin_infos, coin_type);
        table::borrow(&coin_infos.coin_infos, coin_type)
    }

    fun borrow_mut_coin_info<CoinType: key>(ctx: &mut Context): &mut CoinInfo {
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        let coin_type = type_info::type_name<CoinType>();
        check_coin_info_registered_internal(coin_infos, coin_type);
        table::borrow_mut(&mut coin_infos.coin_infos, coin_type)
    }

    fun check_coin_info_registered_internal(coin_infos: &CoinInfos, coin_type: string::String) {
        assert!(
            table::contains(&coin_infos.coin_infos, coin_type),
            error::not_found(ErrorCoinInfoNotRegistered),
        );
    }

    // Unpack the Coin and return the value
    public(friend) fun unpack<CoinType: key>(coin: Coin<CoinType>) : u256 {
        let Coin { value } = coin;
        value
    }

    // Pack the value into Coin
    public(friend) fun pack<CoinType: key>(value: u256) : Coin<CoinType> {
        Coin<CoinType> {
            value
        }
    }
}
