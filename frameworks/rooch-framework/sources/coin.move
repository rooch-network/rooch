// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::string;
    use moveos_std::object::{Self, ObjectID, Object};
    
    use moveos_std::event;
    use moveos_std::type_info::Self;

    friend rooch_framework::genesis;
    friend rooch_framework::coin_store;

    //
    // Errors
    //

    /// `CoinType` is not registered as a coin
    const ErrorCoinInfoNotRegistered: u64 = 1;

    /// `CoinType` is already registered as a coin
    const ErrorCoinInfoAlreadyRegistered: u64 = 2;

    /// Not enough coins to extract
    const ErrorInsufficientBalance: u64 = 3;

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

    // Core data structures

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

    /// Information about a specific coin type. Stored in the global Object storage.
    /// CoinInfo<CoinType> is a named Object, the `coin_type` is the unique key.
    struct CoinInfo<phantom CoinType : key> has key, store {
        /// Type of the coin: `address::module_name::CoinStructName`, same as `moveos_std::type_info::type_name<CoinType>()`.
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

    /// Event emitted when coin minted.
    struct MintEvent has drop, store, copy {
        /// The type of coin that was minted
        coin_type: string::String,
        /// coin amount added to the system
        amount: u256,
    }

    /// Event emitted when coin burned.
    struct BurnEvent has drop, store, copy {
        /// The type of coin that was burned
        coin_type: string::String,
        /// coin amount removed from the system
        amount: u256,
    }

    public(friend) fun genesis_init(__genesis_account: &signer) {}

    //
    // Public functions
    //

    /// A helper function that returns the address of CoinType.
    public fun coin_address<CoinType: key>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }

    /// A helper function that check the `CoinType` is registered, if not, abort.
    public fun check_coin_info_registered<CoinType: key>() {
        assert!(is_registered<CoinType>(), ErrorCoinInfoNotRegistered);
    }

    /// Returns `true` if the type `CoinType` is an registered coin.
    public fun is_registered<CoinType: key>(): bool {
        let object_id = coin_info_id<CoinType>();
        object::exists_object_with_type<CoinInfo<CoinType>>(object_id)
    }

    /// Return the ObjectID of Object<CoinInfo<CoinType>>
    public fun coin_info_id<CoinType: key>(): ObjectID {
        object::named_object_id<CoinInfo<CoinType>>()
    }

    /// Returns the name of the coin.
    public fun name<CoinType: key>(coin_info: &CoinInfo<CoinType>): string::String {
        coin_info.name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType: key>(coin_info: &CoinInfo<CoinType>): string::String {
        coin_info.symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType: key>(coin_info: &CoinInfo<CoinType>): u8 {
        coin_info.decimals
    }

    /// Returns the amount of coin in existence.
    public fun supply<CoinType: key>(coin_info: &CoinInfo<CoinType>): u256 {
        coin_info.supply
    }

    /// Return true if the type `CoinType1` is same with `CoinType2`
    public fun is_same_coin<CoinType1, CoinType2>(): bool {
        return type_info::type_of<CoinType1>() == type_info::type_of<CoinType2>()
    }

    /// Destroys a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Coin`. 
    public fun destroy_zero<CoinType: key>(zero_coin: Coin<CoinType>) {
        let Coin { value } = zero_coin;
        assert!(value == 0, ErrorDestroyOfNonZeroCoin)
    }

    /// Extracts `amount` from the passed-in `coin`, where the original coin is modified in place.
    public fun extract<CoinType: key>(coin: &mut Coin<CoinType>, amount: u256): Coin<CoinType> {
        assert!(coin.value >= amount, ErrorInsufficientBalance);
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

    /// Borrow the CoinInfo<CoinType>
    public fun coin_info<CoinType: key>(): &CoinInfo<CoinType> {
        let coin_info_id = coin_info_id<CoinType>();
        assert!(object::exists_object_with_type<CoinInfo<CoinType>>(coin_info_id), ErrorCoinInfosNotFound);
        let coin_info_obj = object::borrow_object<CoinInfo<CoinType>>(coin_info_id);
        object::borrow(coin_info_obj)
    }

    //
    // Extend functions
    //

    #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType`
    /// This function is protected by `private_generics`, so it can only be called by the `CoinType` module.
    public fun register_extend<CoinType: key>(
        
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ): Object<CoinInfo<CoinType>> {
        assert!(
            !is_registered<CoinType>(),
            ErrorCoinInfoAlreadyRegistered,
        );

        let coin_type = type_info::type_name<CoinType>();

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, ErrorCoinNameTooLong);
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, ErrorCoinSymbolTooLong);

        let coin_info = CoinInfo<CoinType> {
            coin_type,
            name,
            symbol,
            decimals,
            supply: 0u256,
        };
        object::new_named_object(coin_info)
    }

    /// Public coin can mint by anyone with the mutable Object<CoinInfo<CoinType>>
    public fun mint<CoinType: key + store>(coin_info: &mut Object<CoinInfo<CoinType>>, amount: u256): Coin<CoinType> {
        mint_internal(coin_info, amount)
    }

    #[private_generics(CoinType)]
    /// Mint new `Coin`, this function is only called by the `CoinType` module, for the developer to extend custom mint logic
    public fun mint_extend<CoinType: key>(coin_info: &mut Object<CoinInfo<CoinType>>, amount: u256): Coin<CoinType> {
        mint_internal<CoinType>(coin_info, amount)
    }

    /// Public coin can burn by anyone with the mutable Object<CoinInfo<CoinType>>
    public fun burn<CoinType: key + store>(coin_info: &mut Object<CoinInfo<CoinType>>, coin: Coin<CoinType>) {
        burn_internal(coin_info, coin)
    }

    #[private_generics(CoinType)]
    /// Burn `coin`
    /// This function is only called by the `CoinType` module, for the developer to extend custom burn logic
    public fun burn_extend<CoinType: key>(
        coin_info: &mut Object<CoinInfo<CoinType>>,
        coin: Coin<CoinType>,
    ) {
        burn_internal(coin_info, coin)
    }

    //
    // Internal functions
    //

    fun mint_internal<CoinType: key>(coin_info_obj: &mut Object<CoinInfo<CoinType>>,
                                     amount: u256): Coin<CoinType> {
        let coin_info = object::borrow_mut(coin_info_obj);
        coin_info.supply = coin_info.supply + amount;
        let coin_type = type_info::type_name<CoinType>();
        event::emit<MintEvent>(MintEvent {
            coin_type,
            amount,
        });
        Coin<CoinType> { value: amount }
    }

    fun burn_internal<CoinType: key>(
        coin_info_obj: &mut Object<CoinInfo<CoinType>>,
        coin: Coin<CoinType>,
    ) {
        let coin_info = object::borrow_mut(coin_info_obj);
        let Coin { value: amount } = coin;

        let coin_type = type_info::type_name<CoinType>();
        coin_info.supply = coin_info.supply - amount;
        event::emit<BurnEvent>(BurnEvent {
            coin_type,
            amount,
        });
    }

    // Unpack the Coin and return the value
    public(friend) fun unpack<CoinType: key>(coin: Coin<CoinType>): u256 {
        let Coin { value } = coin;
        value
    }

    // Pack the value into Coin
    public(friend) fun pack<CoinType: key>(value: u256): Coin<CoinType> {
        Coin<CoinType> {
            value
        }
    }

    #[test_only]
    public fun destroy_for_testing<CoinType: key>(coin: Coin<CoinType>) {
        let Coin { value:_ } = coin;
    }
}
