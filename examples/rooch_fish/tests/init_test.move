// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_fish::init_test {
    use std::vector;
    use moveos_std::object::{Self, ObjectID};
    use rooch_framework::genesis;
    use rooch_fish::rooch_fish::{Self, GameState};

    struct PondConfig has drop {
        width: u64,
        height: u64,
        max_fish_count: u64,
        purchase_amount: u64,
    }

    #[test(admin = @rooch_fish)]
    fun test_game_initialization(admin: signer) {
        // Initialize the test environment
        genesis::init_for_test();

        // Initialize the game world
        rooch_fish::init_world(&admin);
        
        // Get GameState object
        let game_state_id = object::named_object_id<GameState>();
        let game_state_obj = object::borrow_mut_object_shared<GameState>(game_state_id);

        // Verify the number of ponds
        assert!(rooch_fish::get_pond_count(game_state_obj) == 8, 0);

        // Define expected pond configurations
        let expected_configs = vector[
            PondConfig { width: 100, height: 100, max_fish_count: 100, purchase_amount: 100000 },
            PondConfig { width: 1000, height: 1000, max_fish_count: 1000, purchase_amount: 100000 },
            PondConfig { width: 10000, height: 10000, max_fish_count: 10000, purchase_amount: 100000 },
            PondConfig { width: 100000, height: 100000, max_fish_count: 100000, purchase_amount: 100000 },
            PondConfig { width: 1000, height: 1000, max_fish_count: 1000, purchase_amount: 1000000 },
            PondConfig { width: 1000, height: 1000, max_fish_count: 1000, purchase_amount: 10000000 },
            PondConfig { width: 1000, height: 1000, max_fish_count: 1000, purchase_amount: 100000000 },
            PondConfig { width: 1000, height: 1000, max_fish_count: 1000, purchase_amount: 1000000000 },
        ];

        // Verify each pond's configuration
        let i = 0;
        while (i < 8) {
            let (width, height, max_fish_count, purchase_amount,_,_) = rooch_fish::get_pond_info(game_state_obj, (i as u64));
            let expected_config = vector::borrow(&expected_configs, i);
            
            assert!(width == expected_config.width, 1);
            assert!(height == expected_config.height, 2);
            assert!(max_fish_count == expected_config.max_fish_count, 3);
            assert!(purchase_amount == (expected_config.purchase_amount as u256), 4);

            i = i + 1;
        };

        // Verify global player count and total feed
        assert!(rooch_fish::get_global_player_count(game_state_obj) == 0, 5);
        assert!(rooch_fish::get_global_total_feed(game_state_obj) == 0, 6);
    }
}

