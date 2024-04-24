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
/// 2. After planting the plant, the plant's growth status is stored in the `Inscription` 's permanently state area on Rooch .
///     Once the seed Inscription is transferred on Bitcoin, the plant produced by this seed will also be transferred.
/// 3. After watering the plant, the plant's growth status will be updated. The watering action will also be recorded in the Inscription's temporary state area.
///     Once the seed Inscription is transferred, the watering actions will be cleaned.
/// 4. Once the plants have produced, players can harvest the fruit, the fruit will be minted as a Coin or NFT on Rooch, 
///     which has no more relationship with the seed and the plant.
module bitcoin_plants::plants {

    use std::vector;
    use moveos_std::tx_context;
    use moveos_std::object::{Self, Object};
    use rooch_framework::timestamp;
    use bitcoin_move::ord::{Self, Inscription};

    const ErrorNotSeed: u64 = 0;
    const ErrorAlreadyPlanted: u64 = 1;
    const ErrorNotPlanted: u64 = 2;
    const ErrorWateringTooFrequently: u64 = 3;
    const ErrorPlantDead: u64 = 4;

    const WATERING_INTERVAL: u64 = 60 * 60 * 24; // 1 day

    /// The fruit NFT.
    // TODO: maybe SFT is more suitable to represent the fruits.
    struct Fruits has key, store {
        variety: u64,
        value: u64,
    }

    /// The plant.
    /// Recording the plant's status and will be stored in the permanent state area of the Inscription.
    struct Plant has store {
        /// The variety of the plant. The type variety is inferred from the seed Inscription.
        variety: u64,
        /// The growth value of the plant. The value will be updated by watering actions.
        growth_value: u64,
        /// The health of the plant. Health will decrease if the plant is not watered.
        /// The health will be updated by watering actions.
        /// Health value ranges in [0,100], if health is 0, the plant is dead.
        health: u8,
        /// The last time that the plant has been watered, in seconds.
        last_watering_time: u64,
        /// The number of fruits that are available to be picked.
        pickable_fruits: u64,
        /// The number of fruits that have been picked.
        picked_fruits: u64,
    }

    /// Actions of interaction with the plant.
    struct Actions has store, copy, drop {
        planting_time: u64,
        watering_time: vector<u64>,
        harvest_time: vector<u64>,
        harvest_amount: vector<u64>,
    }

    fun init() {
    }

    /// Plant with seed represented by the Inscription
    // Parse the Inscription and store planting info and plant states in the UTXO's permanent state area.
    public entry fun plant(seed: &mut Object<Inscription>) {
        // Parse the Inscription, check if it is a seed Inscription.
        let inscription = object::borrow(seed);
        ensure_seed_inscription(inscription);

        // Check if the Inscription is already planted
        assert!(!ord::contains_permanent_state<Plant>(seed), ErrorAlreadyPlanted);
        // TODO: init the Plant from seed attributes
        let plant = Plant {
            variety: 0,
            growth_value: 0,
            health: 100,
            last_watering_time: timestamp::now_seconds(),
            pickable_fruits: 0,
            picked_fruits: 0,
        };

        // Store the planting info and plant status in the Inscription's permanent state area
        ord::add_permanent_state(seed, plant);

        // Init and store the planting action in the Inscription's temporary state area
        let actions = Actions {
            planting_time: timestamp::now_seconds(),
            watering_time: vector[],
            harvest_time: vector[],
            harvest_amount: vector[],
        };
        ord::add_temp_state(seed, actions);
    }

    public entry fun water(seed: &mut Object<Inscription>) {
        let plant = ord::borrow_mut_permanent_state<Plant>(seed);
        let watering_interval = timestamp::now_seconds() - plant.last_watering_time;
        assert!(watering_interval >= WATERING_INTERVAL, ErrorWateringTooFrequently);
        // Update plant status and watering actions
        plant.health = calculate_health(plant.growth_value, plant.health, watering_interval);
        if (plant.health == 0) {
            // The plant is dead
            return
        };
        plant.growth_value = plant.growth_value + 1;
        let now = timestamp::now_seconds();
        plant.last_watering_time = now;

        // Update fruits status
        if (plant.growth_value >= 10) {
            plant.pickable_fruits = (plant.growth_value - 10) / 5 + 1 - plant.picked_fruits;
        };
        
        let actions = ord::borrow_mut_temp_state<Actions>(seed);
        vector::push_back(&mut actions.watering_time, now);
    }

    public fun do_harvest(seed: &mut Object<Inscription>): vector<Fruits> {
        let plant = ord::borrow_mut_permanent_state<Plant>(seed);
        assert!(plant.health > 0, ErrorPlantDead);

        // Harvest the plant if there are fruits
        if (plant.pickable_fruits > 0) {
            plant.picked_fruits = plant.picked_fruits + plant.pickable_fruits;
            let fruits = Fruits {
                variety: plant.variety,
                value: plant.pickable_fruits
            };
            plant.pickable_fruits = 0;

            let actions = ord::borrow_mut_temp_state<Actions>(seed);
            vector::push_back(&mut actions.harvest_time, timestamp::now_seconds());
            vector::push_back(&mut actions.harvest_amount, fruits.value);
            vector[fruits]
        } else {
            vector[]
        }
    }

    public entry fun harvest(plant: &mut Object<Inscription>) {
        let fruits = do_harvest(plant);
        if (vector::length(&fruits) > 0) {
            let fruit = vector::pop_back(&mut fruits);
            let obj = object::new(fruit);
            object::transfer(obj, tx_context::sender());
        };
        vector::destroy_empty(fruits);
    }

    public fun is_seed(_inscription: &Inscription) : bool {
        // TODO: Parse the Inscription content and check if it is a seed Inscription
        true
    }

    /// Infer the level of plant through growth value.
    public fun infer_plant_level(_growth_value: u64): u8 {
        // TODO
        1
    }

    /// Calculate the health of the plant.
    /// With longer `thirst_duration`, the health will decrease more.
    /// With higher growth value, the health will decrease slower.
    public fun calculate_health(_growth_value: u64, health: u8, _thirst_duration: u64): u8 {
        // TODO
        health
    }

    fun ensure_seed_inscription(inscription: &Inscription) {
        assert!(is_seed(inscription), ErrorNotSeed);
    }

    #[test_only]
    use std::option;

    #[test_only]
    use rooch_framework::genesis;

    #[test]
    fun test() {
        genesis::init_for_test();
        let inscription_obj = ord::new_inscription_object_for_test(
            @0x3232423,
            0,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            option::none(),
            option::none(),
        );

        plant(&mut inscription_obj);

        let i = 0u8;
        loop {
            timestamp::fast_forward_seconds_for_test(WATERING_INTERVAL);
            water(&mut inscription_obj);
            i = i + 1;
            if (i == 10) break;
        };

        let fruits = do_harvest(&mut inscription_obj);
        assert!(vector::length(&fruits) == 1, 1);
        let Fruits { variety: _, value: _} = vector::pop_back(&mut fruits);
        vector::destroy_empty(fruits);

        let plant = ord::remove_permanent_state<Plant>(&mut inscription_obj);
        let Plant { variety: _, growth_value: _, health: _, last_watering_time: _, pickable_fruits: _, picked_fruits: _ } = plant;
        ord::destroy_permanent_area(&mut inscription_obj);
        ord::drop_temp_area_for_test(&mut inscription_obj);
        ord::drop_inscription_object_for_test(inscription_obj);
    }
}