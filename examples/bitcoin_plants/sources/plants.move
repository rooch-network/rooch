// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// A game example to show how to use the `Inscription` and `UTXO` in Rooch.
///
/// The game is about planting a plant from seed(Inscription),  then watering and harvesting.
/// The full game flow and main actions are as follows:
/// 1. Mint seed: players mint seed Inscription on Bitcoin.
/// 2. Plant: Using the seed inscription, players plant plants on Rooch.
/// 3. Water: Players water the plant on Rooch Layer 2 regularly. The seeds will grow, sprout, and bear fruit.    
/// 4. Harvest:  Players harvest the fruits. The fruit is represented by Layer2's Coin or NFT.
///
/// In this example, you will learn how Rooch is acting as a Layer 2 to interact with Bitcoin Layer 1.
/// The details are as follows:
/// 1. All Bitcoin transactions are processed and tracked on Rooch. All UTXOs are represented by the `UTXO` object on Rooch.
///     You can read all Bitcoin states. So if you have a seed Inscription on Bitcoin, you can know that.
/// 2. After planting the plant, the plant's growth status is stored in the `Inscription` UTXO's permanently state area on Rooch .
///     Once the seed Inscription is transferred on Bitcoin, the plant produced by this seed will also be transferred.
/// 3. After watering the plant, the plant's growth status will be updated. The watering action will also be recorded in the `Inscription` UTXO's temporary state area.
///     Once the seed Inscription is transferred on Bitcoin, the watering actions will be cleaned.
/// 4. Once the plants have produced, players can harvest the fruit, the fruit will be minted as a Coin or NFT on Rooch, 
///     which has no more relationship with the seed and the plant.
module bitcoin_plants::plants {

    use std::string;
    use moveos_std::tx_context;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;
    use rooch_framework::timestamp;
    use bitcoin_move::utxo::{Self, UTXO};
    use bitcoin_move::ord::Inscription;

    const ErrorAlreadyStaked: u64 = 1;
    const ErrorAlreadyClaimed: u64 = 2;

    /// The fruit NFT.
    struct Fruit has key, store {
        variety: u64,
        value: u64,
    }

    /// The plant.
    /// Recording the plant's status and will be stored in the permanent state area of the UTXO
    struct Plant has store {
        /// The variety of the plant. The type variety is inferred from the seed Inscription.
        variety: u64,
        /// The growth value of the plant. The value will be updated by watering actions.
        /// All status can be inferred from the growth value.
        growth_value: u64,
        status: u8,
        last_watering_time: u64,
    }

    fun init() {
        let coin_info_obj = coin::register_extend<HDC>(
            string::utf8(b"BTC Holder Coin"),
            string::utf8(b"HDC"),
            DECIMALS,
        );
        let coin_info_holder_obj = object::new_named_object(CoinInfoHolder { coin_info: coin_info_obj });
        // Make the coin info holder object to shared, so anyone can get mutable CoinInfoHolder object
        object::to_shared(coin_info_holder_obj);
    }

    /// Plant with seed represented by the Inscription
    // Parse the Inscription and store planting info and plant states in the UTXO's permanent state area.
    public fun do_plant(seed: &mut Object<Inscription>) {
        // Parse the Inscription, check if it is a seed Inscription.
        // A seed inscription has protocol `bitseed` and tick name `seed`


        // Check if the UTXO is already planted

        // Store the planting info and plant status in the UTXO's permanent state area
    }

    public fun do_water(plant: &mut Object<Inscription>) {
        // Check watering actions of this plant

        // Update plant status and watering actions
    }

    public fun do_harvest(plant: &mut Object<Inscription>) {
        // Check the plant's status.

        // Harvest the plant if there are fruits
    }

    public fun is_seed(json_map: &SimpleMap<String,String>) : bool {
        let protocol_key = string::utf8(b"p");
        simple_map::contains_key(json_map, &protocol_key) && \
        simple_map::borrow(json_map, &protocol_key) == &string::utf8(b"bitseed") && \
        simple_map::borrow(json_map, &string::utf8(b"n")) == &string::utf8(b"seed")
    }
}