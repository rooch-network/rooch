// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module provides the foundation for typesafe Generic Coins.
module rooch_framework::generic_coin {
    use std::string;

    friend rooch_framework::coin;
    friend rooch_framework::multi_coin_store;

    //
    // Errors
    //

    /// The coin type is not match
    const ErrorCoinTypeNotMatch: u64 = 1;


    // Core data structures

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
    public fun unpack_generic_coin_for_test(coin: GenericCoin): (string::String, u256) {
        unpack_generic_coin(coin)
    }
}
