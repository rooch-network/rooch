// tests/fish_lifecycle_test.move
module rooch_fish::fish_lifecycle_test {
    use std::signer;
    use std::vector;
    use moveos_std::object;

    use rooch_framework::genesis;
    use rooch_framework::gas_coin;

    use rooch_fish::player;
    use rooch_fish::rooch_fish::{Self, GameState};

    const POND_ID: u64 = 0;
    const INITIAL_BALANCE: u256 = 1000000000000; // 10000 RGAS

    #[test(admin = @rooch_fish, player = @0x42)]
    fun test_fish_lifecycle(admin: signer, player: signer) {
        // Initialize the test environment
        genesis::init_for_test();
        rooch_fish::init_world(&admin);

        let player_addr = signer::address_of(&player);
        gas_coin::faucet_for_test(player_addr, INITIAL_BALANCE);

        // Get GameState object
        let game_state_id = object::named_object_id<GameState>();
        let game_state_obj = object::borrow_mut_object_shared<GameState>(game_state_id);

        // Purchase a fish
        rooch_fish::purchase_fish(&player, game_state_obj, POND_ID);
        
        // Verify fish count increased
        assert!(rooch_fish::get_pond_player_count(game_state_obj, POND_ID) == 1, 1);
        
        // Get the fish ID
        let fish_ids = rooch_fish::get_pond_player_fish_ids(game_state_obj, POND_ID, player_addr);
        assert!(vector::length(&fish_ids) == 1, 2);
        let fish_id = *vector::borrow(&fish_ids, 0);
        
        // Set fish position for testing
        rooch_fish::set_fish_position_for_test(game_state_obj, POND_ID, fish_id, 25, 25);
        
        // Move the fish
        rooch_fish::move_fish(&player, game_state_obj, POND_ID, fish_id, 1); // Move right
        
        // Feed food
        let food_count = 5;
        rooch_fish::feed_food(&player, game_state_obj, POND_ID, food_count);
        
        // Get last food ID and set its position near the fish
        let last_food_id = rooch_fish::get_last_food_id(game_state_obj, POND_ID);
        rooch_fish::set_food_position_for_test(game_state_obj, POND_ID, last_food_id, 26, 25);
        
        // Verify total feed increased (exact amount depends on pond config)
        assert!(rooch_fish::get_pond_total_feed(game_state_obj, POND_ID) > 0, 3);
        
        // Set fish position to exit zone for testing
        rooch_fish::set_fish_position_for_test(game_state_obj, POND_ID, fish_id, 50, 50);
        
        // Destroy the fish
        rooch_fish::destroy_fish(&player, game_state_obj, POND_ID, fish_id);
        
        // Verify fish count decreased
        let fish_ids = rooch_fish::get_pond_player_fish_ids(game_state_obj, POND_ID, player_addr);
        assert!(vector::length(&fish_ids) == 0, 4);

        // Verify player received rewards
        let player_list = rooch_fish::get_global_player_list(game_state_obj);
        let player_reward = player::get_player_reward(player_list, player_addr);
        assert!(player_reward > 0, 5);
    }
}
