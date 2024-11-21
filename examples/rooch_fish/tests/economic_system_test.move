// tests/economic_system_test.move
module rooch_fish::economic_system_test {
    use std::signer;
    use std::vector;
    use moveos_std::object;

    use rooch_framework::genesis;
    use rooch_framework::gas_coin;
    use rooch_fish::rooch_fish::{Self, GameState};
    use rooch_fish::player;

    const POND_ID_SMALL: u64 = 0;  // Assuming pond 0 is the smallest
    const POND_ID_LARGE: u64 = 7;  // Assuming pond 7 is the largest
    const INITIAL_BALANCE: u256 = 1000000000000; // 10000 RGAS

    #[test(admin = @rooch_fish, player1 = @0x42, player2 = @0x43)]
    fun test_economic_system(admin: signer, player1: signer, player2: signer) {
        // Initialize the test environment
        genesis::init_for_test();
        rooch_fish::init_world(&admin);

        let player1_addr = signer::address_of(&player1);
        let player2_addr = signer::address_of(&player2);
        gas_coin::faucet_for_test(player1_addr, INITIAL_BALANCE);
        gas_coin::faucet_for_test(player2_addr, INITIAL_BALANCE);

        // Get GameState object
        let game_state_id = object::named_object_id<GameState>();
        let game_state_obj = object::borrow_mut_object_shared<GameState>(game_state_id);

        // Test purchase in different ponds
        let initial_balance = gas_coin::balance(player1_addr);
        rooch_fish::purchase_fish(&player1, game_state_obj, POND_ID_SMALL);
        let cost_small = initial_balance - gas_coin::balance(player1_addr);

        let initial_balance = gas_coin::balance(player1_addr);
        rooch_fish::purchase_fish(&player1, game_state_obj, POND_ID_LARGE);
        let cost_large = initial_balance - gas_coin::balance(player1_addr);

        // Verify that larger pond costs more
        assert!(cost_large > cost_small, 1);

        // Get fish IDs
        let fish_ids_small = rooch_fish::get_pond_player_fish_ids(game_state_obj, POND_ID_SMALL, player1_addr);
        let fish_ids_large = rooch_fish::get_pond_player_fish_ids(game_state_obj, POND_ID_LARGE, player1_addr);
        let fish1_small = *vector::borrow(&fish_ids_small, 0);
        let fish1_large = *vector::borrow(&fish_ids_large, 0);

        // Test feeding
        let food_count = 5;
        let initial_balance = gas_coin::balance(player1_addr);
        rooch_fish::feed_food(&player1, game_state_obj, POND_ID_SMALL, food_count);
        let feed_cost = initial_balance - gas_coin::balance(player1_addr);
        
        // Get pond info
        let (_, _, _, purchase_amount, _, food_value_ratio) = rooch_fish::get_pond_info(game_state_obj, POND_ID_SMALL);
        
        // Calculate expected cost
        let food_value = purchase_amount / (food_value_ratio as u256);
        let expected_cost = (food_count as u256) * food_value;
        
        // Assert that the actual cost matches the expected cost
        assert!(feed_cost == expected_cost, 2);

        // Test fish growth and reward
        let food_id = rooch_fish::get_last_food_id(game_state_obj, POND_ID_SMALL);
        rooch_fish::set_food_position_for_test(game_state_obj, POND_ID_SMALL, food_id, 26, 25);
        rooch_fish::set_fish_position_for_test(game_state_obj, POND_ID_SMALL, fish1_small, 25, 25);
        rooch_fish::move_fish(&player1, game_state_obj, POND_ID_SMALL, fish1_small, 1);

        rooch_fish::set_fish_position_for_test(game_state_obj, POND_ID_SMALL, fish1_small, 50, 50);
        let initial_balance = gas_coin::balance(player1_addr);
        rooch_fish::destroy_fish(&player1, game_state_obj, POND_ID_SMALL, fish1_small);
        let reward_small = gas_coin::balance(player1_addr) - initial_balance;

        // Test reward in larger pond
        rooch_fish::set_fish_position_for_test(game_state_obj, POND_ID_LARGE, fish1_large, 500, 500);
        let initial_balance = gas_coin::balance(player1_addr);
        rooch_fish::destroy_fish(&player1, game_state_obj, POND_ID_LARGE, fish1_large);
        let reward_large = gas_coin::balance(player1_addr) - initial_balance;

        assert!(reward_large > reward_small, 3);
        
        // Test feeding distribution
        rooch_fish::purchase_fish(&player2, game_state_obj, POND_ID_SMALL);
        rooch_fish::feed_food(&player1, game_state_obj, POND_ID_SMALL, food_count);

        let player_list = rooch_fish::get_global_player_list(game_state_obj);
        let player1_feed = player::get_player_feed_amount(player_list, player1_addr);
        let player2_feed = player::get_player_feed_amount(player_list, player2_addr);

        // Get pond info for final verification
        let (_, _, _, purchase_amount, _, food_value_ratio) = rooch_fish::get_pond_info(game_state_obj, POND_ID_SMALL);
        let food_value = purchase_amount / (food_value_ratio as u256);
        let expected_total_feed = (food_count as u256) * food_value * 2; // Player 1 fed twice

        assert!(player1_feed == expected_total_feed, 4);
        assert!(player2_feed == 0, 5);

        // Clean up
        let fish_ids_small = rooch_fish::get_pond_player_fish_ids(game_state_obj, POND_ID_SMALL, player2_addr);
        let fish2_small = *vector::borrow(&fish_ids_small, 0);
        rooch_fish::set_fish_position_for_test(game_state_obj, POND_ID_SMALL, fish2_small, 50, 50);
        rooch_fish::destroy_fish(&player2, game_state_obj, POND_ID_SMALL, fish2_small);
    }
}

