// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::option;
    use std::option::Option;
    use std::string;
    use std::string::String;
    use moveos_std::object::{Self, ObjectID, Object};

    use moveos_std::event;
    use moveos_std::type_info::{Self};

    friend rooch_framework::genesis;
    friend rooch_framework::coin_store;
    friend rooch_framework::multi_coin_store;
    friend rooch_framework::account_coin_store;

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

    /// CoinRegister is already initialized
    const ErrorCoinRegisterAlreadyInitialized: u64 = 9;

    /// The function is deprecated
    const ErrorDeprecated: u64 = 10;

    /// The coin type is not match
    const ErrorCoinTypeNotMatch: u64 = 11;

    /// The coin type is invalid
    const ErrorCoinTypeInvalid: u64 = 12;

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

    /// Main structure representing a coin.
    /// Note the `CoinType` must have `key` ability.
    /// if the `CoinType` has `store` ability, the `Coin` is a public coin, the user can operate it directly by coin module's function.
    /// Otherwise, the `Coin` is a private coin, the user can only operate it by `CoinType` module's function.
    /// The Coin has no ability, it is a hot potato type, only can handle by Coin module.
    struct GenericCoin {
        /// Coin type name
        coin_type: string::String,
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
        /// Icon url of the coin
        icon_url: Option<string::String>,
        /// Number of decimals used to get its user representation.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        /// The total value for the coin represented by coin type. Mutable.
        supply: u256,
    }

    /// Coin metadata is copied from CoinInfo, and stored as dynamic field of CoinRegistry
    struct CoinMetadata has key, store {
        coin_info_id: ObjectID,
        coin_type: string::String,
        name: string::String,
        symbol: string::String,
        icon_url: Option<string::String>,
        decimals: u8,
        supply: u256,
    }

    /// The registry of all coin types.
    struct CoinRegistry has key {}

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

    public(friend) fun genesis_init(__genesis_account: &signer) {
        init_coin_registry();
    }

    /// Initialize the CoinRegistry, this function is for framework upgrade.
    entry fun init_coin_registry() {
        let coin_registry_id = object::named_object_id<CoinRegistry>();
        assert!(!object::exists_object_with_type<CoinRegistry>(coin_registry_id), ErrorCoinRegisterAlreadyInitialized);
        let coin_registry = object::new_named_object(CoinRegistry {});
        object::transfer_extend(coin_registry, @rooch_framework);
    }

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

    /// Returns the coin info id by the coin type name
    public fun coin_info_id_by_type_name(coin_type: string::String): ObjectID {
        let registry = borrow_registry();
        assert!(object::contains_field(registry, coin_type), ErrorCoinInfoNotRegistered);
        let coin_metadata: &CoinMetadata = object::borrow_field(registry, coin_type);
        coin_metadata.coin_info_id
    }

    /// Returns the name of the coin.
    public fun name<CoinType: key>(coin_info: &CoinInfo<CoinType>): string::String {
        coin_info.name
    }

    /// Returns the name of the coin by the type `CoinType`
    public fun name_by_type<CoinType: key>(): string::String {
        let coin_type = type_info::type_name<CoinType>();
        name_by_type_name(&coin_type)
    }

    /// Returns the name of the coin by the coin type name
    public fun name_by_type_name(coin_type_name: &String): string::String {
        let registry = borrow_registry();
        assert!(object::contains_field(registry, *coin_type_name), ErrorCoinInfoNotRegistered);
        let coin_metadata: &CoinMetadata = object::borrow_field(registry, *coin_type_name);
        coin_metadata.name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType: key>(coin_info: &CoinInfo<CoinType>): string::String {
        coin_info.symbol
    }

    /// Returns the symbol of the coin by the type `CoinType`
    public fun symbol_by_type<CoinType: key>(): string::String {
        let coin_type = type_info::type_name<CoinType>();
        symbol_by_type_name(&coin_type)
    }

    /// Returns the symbol of the coin by the coin type name
    public fun symbol_by_type_name(coin_type_name: &String): string::String {
        let registry = borrow_registry();
        assert!(object::contains_field(registry, *coin_type_name), ErrorCoinInfoNotRegistered);
        let coin_metadata: &CoinMetadata = object::borrow_field(registry, *coin_type_name);
        coin_metadata.symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType: key>(coin_info: &CoinInfo<CoinType>): u8 {
        coin_info.decimals
    }

    /// Returns the decimals of the coin by the type `CoinType`
    public fun decimals_by_type<CoinType: key>(): u8 {
        let coin_type = type_info::type_name<CoinType>();
        decimals_by_type_name(&coin_type)
    }

    /// Returns the decimals of the coin by the coin type name
    public fun decimals_by_type_name(coin_type_name: &String): u8 {
        let registry = borrow_registry();
        assert!(object::contains_field(registry, *coin_type_name), ErrorCoinInfoNotRegistered);
        let coin_metadata: &CoinMetadata = object::borrow_field(registry, *coin_type_name);
        coin_metadata.decimals
    }

    /// Returns the amount of coin in existence.
    public fun supply<CoinType: key>(coin_info: &CoinInfo<CoinType>): u256 {
        coin_info.supply
    }

    /// Returns the amount of coin in existence by the type `CoinType`
    public fun supply_by_type<CoinType: key>(): u256 {
        let coin_type = type_info::type_name<CoinType>();
        supply_by_type_name(&coin_type)
    }

    /// Returns the amount of coin in existence by the coin type name
    public fun supply_by_type_name(coin_type_name: &String): u256 {
        let registry = borrow_registry();
        assert!(object::contains_field(registry, *coin_type_name), ErrorCoinInfoNotRegistered);
        let coin_metadata: &CoinMetadata = object::borrow_field(registry, *coin_type_name);
        coin_metadata.supply
    }

    /// Returns the icon url of coin.
    public fun icon_url<CoinType: key>(coin_info: &CoinInfo<CoinType>): Option<String> {
        coin_info.icon_url
    }

    /// Returns the icon url of coin by the type `CoinType`
    public fun icon_url_by_type<CoinType: key>(): Option<String> {
        let coin_type = type_info::type_name<CoinType>();
        icon_url_by_type_name(&coin_type)
    }

    /// Returns the icon url of the coin by the coin type name
    public fun icon_url_by_type_name(coin_type_name: &String): Option<String> {
        let registry = borrow_registry();
        assert!(object::contains_field(registry, *coin_type_name), ErrorCoinInfoNotRegistered);
        let coin_metadata: &CoinMetadata = object::borrow_field(registry, *coin_type_name);
        coin_metadata.icon_url
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

    // Add this function to help with dynamic coin operations
    public fun get_coin_info_by_type_name(coin_type_name: &String): Option<ObjectID> {
        let registry = borrow_registry();
        if (!object::contains_field(registry, *coin_type_name)) {
            option::none()
        } else {
            let metadata: &CoinMetadata = object::borrow_field(registry, *coin_type_name);
            option::some(metadata.coin_info_id)
        }
    }

    //
    // Extend functions
    //

    #[private_generics(CoinType)]
    /// This function is protected by `private_generics`, so it can only be called by the `CoinType` module.
    public fun upsert_icon_url<CoinType: key>(coin_info_obj: &mut Object<CoinInfo<CoinType>>, icon_url: String) {
        upsert_icon_url_internal(coin_info_obj, icon_url);
    }


    #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType`
    /// This function is protected by `private_generics`, so it can only be called by the `CoinType` module.
    public fun register_extend<CoinType: key>(
        name: string::String,
        symbol: string::String,
        icon_url: Option<string::String>,
        decimals: u8,
    ): Object<CoinInfo<CoinType>> {
        register_internal<CoinType>(name, symbol, icon_url, decimals)
    }

    /// This function for the old code to initialize the CoinMetadata
    public fun init_metadata<CoinType: key>(coin_info: &Object<CoinInfo<CoinType>>) {
        let _coin_metadata: &mut CoinMetadata = borrow_mut_coin_metadata<CoinType>(coin_info);
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

    fun register_internal<CoinType: key>(
        name: string::String,
        symbol: string::String,
        icon_url: Option<string::String>,
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
            icon_url,
            decimals,
            supply: 0u256,
        };
        let coin_info_obj = object::new_named_object(coin_info);
        let coin_info_id = object::id(&coin_info_obj);

        let coin_metadata = CoinMetadata {
            coin_info_id,
            coin_type,
            name,
            symbol,
            icon_url,
            decimals,
            supply: 0u256,
        };
        let coin_registry = borrow_mut_registry();
        object::add_field(coin_registry, coin_type, coin_metadata);

        coin_info_obj
    }

    fun mint_internal<CoinType: key>(coin_info_obj: &mut Object<CoinInfo<CoinType>>, amount: u256): Coin<CoinType> {
        let coin_type = type_info::type_name<CoinType>();

        let coin_info = object::borrow_mut(coin_info_obj);
        coin_info.supply = coin_info.supply + amount;

        let _coin_metadata: &mut CoinMetadata = borrow_mut_coin_metadata<CoinType>(coin_info_obj);
        // We sync the supply with coin info, so we don't need to update the supply here
        // After we remove the sync, we need to update the supply here
        //coin_metadata.supply = coin_metadata.supply + amount;

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
        let coin_type = type_info::type_name<CoinType>();

        let coin_info = object::borrow_mut(coin_info_obj);
        let Coin { value: amount } = coin;

        coin_info.supply = coin_info.supply - amount;

        let _coin_metadata: &mut CoinMetadata = borrow_mut_coin_metadata<CoinType>(coin_info_obj);
        // We sync the supply with coin info, so we don't need to update the supply here
        // After we remove the sync, we need to update the supply here
        //coin_metadata.supply = coin_metadata.supply - amount;

        event::emit<BurnEvent>(BurnEvent {
            coin_type,
            amount,
        });
    }

    fun upsert_icon_url_internal<CoinType: key>(coin_info_obj: &mut Object<CoinInfo<CoinType>>, icon_url: String) {
        object::borrow_mut(coin_info_obj).icon_url = option::some(icon_url);
        let coin_metadata: &mut CoinMetadata = borrow_mut_coin_metadata<CoinType>(coin_info_obj);
        coin_metadata.icon_url = option::some(icon_url);
    }

    fun borrow_registry(): &Object<CoinRegistry> {
        object::borrow_object<CoinRegistry>(object::named_object_id<CoinRegistry>())
    }

    fun borrow_mut_registry(): &mut Object<CoinRegistry> {
        object::borrow_mut_object_extend<CoinRegistry>(object::named_object_id<CoinRegistry>())
    }

    fun borrow_mut_coin_metadata<CoinType: key>(coin_info_obj: &Object<CoinInfo<CoinType>>): &mut CoinMetadata {
        let coin_type = type_info::type_name<CoinType>();

        let registry = borrow_mut_registry();
        let coin_info_id = object::id(coin_info_obj);
        let coin_info = object::borrow(coin_info_obj);
        // If the coin metadata is not initialized, it is the Coin registered before v19
        // We need to initialize the coin metadata here
        if (!object::contains_field(registry, coin_type)) {
            let coin_metadata = CoinMetadata {
                coin_info_id,
                coin_type,
                name: coin_info.name,
                symbol: coin_info.symbol,
                icon_url: coin_info.icon_url,
                decimals: coin_info.decimals,
                supply: coin_info.supply,
            };
            object::add_field(registry, coin_type, coin_metadata);
            object::borrow_mut_field(registry, coin_type)
        }else {
            //sync the coin metadata with coin info
            let metadata: &mut CoinMetadata = object::borrow_mut_field(registry, coin_type);
            metadata.coin_info_id = coin_info_id;
            metadata.icon_url = coin_info.icon_url;
            metadata.name = coin_info.name;
            metadata.symbol = coin_info.symbol;
            metadata.decimals = coin_info.decimals;
            metadata.supply = coin_info.supply;
            metadata
        }
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
    public fun init_for_testing() {
        let system_signer = moveos_std::account::create_signer_for_testing(@rooch_framework);
        genesis_init(&system_signer);
    }

    #[test_only]
    public fun destroy_for_testing<CoinType: key>(coin: Coin<CoinType>) {
        let Coin { value: _ } = coin;
    }


    // === Migration functions ===
    public fun convert_coin_to_generic_coin<CoinType: key>(coin: Coin<CoinType>): GenericCoin {
        let value = unpack(coin);
        let coin_type = type_info::type_name<CoinType>();
        GenericCoin { coin_type, value }
    }

    public fun convert_generic_coin_to_coin<CoinType: key>(coin: GenericCoin): Coin<CoinType> {
        let generic_coin_type = type_info::type_name<CoinType>();
        let GenericCoin { coin_type, value } = coin;
        assert!(generic_coin_type == coin_type, ErrorCoinTypeNotMatch);
        pack<CoinType>(value)
    }

    // === Non-generic functions ===
    public fun check_coin_info_registered_by_type_name(coin_type: string::String) {
        assert!(is_registered_by_type_name(coin_type), ErrorCoinInfoNotRegistered);
    }

    //
    public fun is_registered_by_type_name(coin_type: string::String): bool {
        let registry = borrow_registry();
        object::contains_field(registry, coin_type)
    }

    // public fun value_by_type(coin_type: string::String, coin: &GenericCoin): u256 {
    public fun generic_coin_value(coin: &GenericCoin): u256 {
        coin.value
    }

    public(friend) fun unpack_generic_coin(coin: GenericCoin): (string::String, u256) {
        let GenericCoin { coin_type, value } = coin;
        (coin_type, value)
    }

    public(friend) fun pack_generic_coin(coin_type: string::String, value: u256): GenericCoin {
        GenericCoin {
            coin_type,
            value
        }
    }

    /// "Merges" the two given generic coins.  The coin passed in as `dst_coin` will have a value equal
    /// to the sum of the two generic coins (`dst_coin` and `source_coin`).
    public fun merge_generic(dst_coin: &mut GenericCoin, source_coin: GenericCoin) {
        let GenericCoin { coin_type: source_coin_type, value: source_value } = source_coin;
        assert!(dst_coin.coin_type == source_coin_type, ErrorCoinTypeNotMatch);
        dst_coin.value = dst_coin.value + source_value;
    }

    /// Helper function for getting the coin type name from a GenericCoin
    public fun coin_type(coin: &GenericCoin): string::String {
        coin.coin_type
    }

    #[test_only]
    public fun pack_for_test<CoinType: key>(value: u256): Coin<CoinType> {
        pack<CoinType>(value)
    }

    #[test_only]
    public fun unpack_for_test<CoinType: key>(coin: Coin<CoinType>): u256 {
        unpack(coin)
    }

    #[test_only]
    public fun unpack_generic_coin_for_test(coin: GenericCoin): (string::String, u256) {
        unpack_generic_coin(coin)
    }

    #[test_only]
    public fun register_for_test<CoinType: key>(
        name: string::String,
        symbol: string::String,
        icon_url: Option<string::String>,
        decimals: u8,
    ): Object<CoinInfo<CoinType>> {
        register_internal<CoinType>(name, symbol, icon_url, decimals)
    }
}
