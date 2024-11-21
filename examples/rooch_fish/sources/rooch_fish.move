// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_fish::rooch_fish {
    use std::vector;
    use std::u256;
    use moveos_std::object::{Self, Object};
    use moveos_std::signer;
    use moveos_std::table::{Self, Table};
    use rooch_framework::gas_coin;
    use rooch_fish::pond::{Self, PondState};
    use rooch_fish::player::{Self, PlayerList};

    const ErrorInvalidPondID: u64 = 1;
    const ErrorAlreadyInitialized: u64 = 2;

    struct PondConfig has copy, drop {
        id: u64,
        owner: address,
        width: u64,
        height: u64,
        purchase_amount: u256,
        max_fish_count: u64,
        max_food_count: u64,
    }

    struct GameState has key {
        admin: address,
        ponds: Table<u64, Object<PondState>>,
        player_list: PlayerList,
    }

    public entry fun init_world(account: &signer) {
        let admin = signer::address_of(account);
        
        let ponds = table::new();
        let unit = u256::pow(10, gas_coin::decimals() - 3);

        let pond_configs = vector[
            PondConfig { id: 0, owner: admin, width: 100, height: 100, purchase_amount: unit, max_fish_count: 100, max_food_count: 1000 },
            PondConfig { id: 1, owner: admin, width: 1000, height: 1000, purchase_amount: unit, max_fish_count: 1000, max_food_count: 10000 },
            PondConfig { id: 2, owner: admin, width: 10000, height: 10000, purchase_amount: unit, max_fish_count: 10000, max_food_count: 100000 },
            PondConfig { id: 3, owner: admin, width: 100000, height: 100000, purchase_amount: unit, max_fish_count: 100000, max_food_count: 1000000 },

            PondConfig { id: 4, owner: admin, width: 1000, height: 1000, purchase_amount: unit * 10, max_fish_count: 1000, max_food_count: 10000 },
            PondConfig { id: 5, owner: admin, width: 1000, height: 1000, purchase_amount: unit * 100, max_fish_count: 1000, max_food_count: 10000 },
            PondConfig { id: 6, owner: admin, width: 1000, height: 1000, purchase_amount: unit * 1000, max_fish_count: 1000, max_food_count: 10000 },
            PondConfig { id: 7, owner: admin, width: 1000, height: 1000, purchase_amount: unit * 10000, max_fish_count: 1000, max_food_count: 10000 },
        ];

        let i = 0;
        while (i < vector::length(&pond_configs)) {
            let config = vector::borrow(&pond_configs, i);
            let pond_obj = pond::create_pond(
                config.id,
                config.owner,
                config.width, 
                config.height, 
                (config.purchase_amount as u256), 
                config.max_fish_count, 
                config.max_food_count,
            );

            let pond_state = object::borrow_mut(&mut pond_obj);
            pond::add_exit_zone(pond_state, config.width/2, config.height/2, 10);

            table::add(&mut ponds, config.id, pond_obj);
            i = i + 1;
        };

        let player_list = player::create_player_list();

        let game_state = GameState {
            admin,
            ponds,
            player_list,
        };

        let game_state_obj = object::new_named_object(game_state);
        object::to_shared(game_state_obj);
    }

    public entry fun purchase_fish(account: &signer, game_state_obj: &mut Object<GameState>, pond_id: u64) {
        let game_state = object::borrow_mut(game_state_obj);
        let account_addr = signer::address_of(account);

        let pond_obj = table::borrow_mut(&mut game_state.ponds, pond_id);
        let pond_state = object::borrow_mut(pond_obj);

        let fish_id = pond::purchase_fish(pond_state, account);
        player::add_fish(&mut game_state.player_list, account_addr, fish_id);
    }

    public entry fun move_fish(account: &signer, game_state_obj: &mut Object<GameState>, pond_id: u64, fish_id: u64, direction: u8) {
        let game_state = object::borrow_mut(game_state_obj);

        let pond_obj = table::borrow_mut(&mut game_state.ponds, pond_id);
        let pond_state = object::borrow_mut(pond_obj);

        pond::move_fish(pond_state, account, fish_id, direction);
    }

    public entry fun feed_food(account: &signer, game_state_obj: &mut Object<GameState>, pond_id: u64, count: u64) {
        let game_state = object::borrow_mut(game_state_obj);
        let account_addr = signer::address_of(account);

        let pond_obj = table::borrow_mut(&mut game_state.ponds, pond_id);
        let pond_state = object::borrow_mut(pond_obj);

        let actual_cost = pond::feed_food(pond_state, account, count);
        player::add_feed(&mut game_state.player_list, account_addr, actual_cost);
    }

    public entry fun destroy_fish(account: &signer, game_state_obj: &mut Object<GameState>, pond_id: u64, fish_id: u64) {
        let game_state = object::borrow_mut(game_state_obj);
        let account_addr = signer::address_of(account);

        let pond_obj = table::borrow_mut(&mut game_state.ponds, pond_id);
        let pond_state = object::borrow_mut(pond_obj);
        
        let reward = pond::destroy_fish(pond_state, account, fish_id);

        let reward_amount = u256::divide_and_round_up(reward, u256::pow(10, gas_coin::decimals()));
        player::add_reward(&mut game_state.player_list, account_addr, reward_amount);
    }

    public fun get_pond_player_list(game_state_obj: &Object<GameState>, pond_id: u64): &PlayerList {
        let game_state = object::borrow(game_state_obj);
        let pond_obj = table::borrow(&game_state.ponds, pond_id);
        let pond_state = object::borrow(pond_obj);
        pond::get_player_list(pond_state)
    }

    public fun get_pond_player_count(game_state_obj: &Object<GameState>, pond_id: u64): u64 {
        let game_state = object::borrow(game_state_obj);
        let pond_obj = table::borrow(&game_state.ponds, pond_id);
        let pond_state = object::borrow(pond_obj);
        pond::get_player_count(pond_state)
    }

    public fun get_pond_total_feed(game_state_obj: &Object<GameState>, pond_id: u64): u256 {
        let game_state = object::borrow(game_state_obj);
        let pond_obj = table::borrow(&game_state.ponds, pond_id);
        let pond_state = object::borrow(pond_obj);
        pond::get_total_feed(pond_state)
    }

    public fun get_pond_player_fish_ids(game_state_obj: &Object<GameState>, pond_id: u64, owner: address): vector<u64> {
        let game_state = object::borrow(game_state_obj);
        let pond_obj = table::borrow(&game_state.ponds, pond_id);
        let pond_state = object::borrow(pond_obj);
        pond::get_player_fish_ids(pond_state, owner)
    }

    public fun get_global_player_list(game_state_obj: &Object<GameState>): &PlayerList {
        let game_state = object::borrow(game_state_obj);
        &game_state.player_list
    }

    public fun get_global_player_count(game_state_obj: &Object<GameState>): u64 {
        let game_state = object::borrow(game_state_obj);
        player::get_player_count(&game_state.player_list)
    }

    public fun get_global_total_feed(game_state_obj: &Object<GameState>): u256 {
        let game_state = object::borrow(game_state_obj);
        player::get_total_feed(&game_state.player_list)
    }

    public fun get_pond_count(game_state_obj: &Object<GameState>): u64 {
        let game_state = object::borrow(game_state_obj);
        table::length(&game_state.ponds)
    }

    public fun get_pond_info(game_state_obj: &Object<GameState>, pond_id: u64): (u64, u64, u64, u256, u64, u64) {
        let game_state = object::borrow(game_state_obj);
        assert!(table::contains(&game_state.ponds, pond_id), ErrorInvalidPondID);
        let pond_obj = table::borrow(&game_state.ponds, pond_id);
        let pond_state = object::borrow(pond_obj);
        (
            pond::get_width(pond_state),
            pond::get_height(pond_state),
            pond::get_max_fish_count(pond_state),
            pond::get_purchase_amount(pond_state),
            pond::get_max_food_per_feed(),
            pond::get_food_value_ratio(),
        )
    }

    #[test_only]
    public fun set_fish_position_for_test(game_state_obj: &mut Object<GameState>, pond_id: u64, fish_id: u64, x: u64, y: u64) {
        let game_state = object::borrow_mut(game_state_obj);
        let pond_obj = table::borrow_mut(&mut game_state.ponds, pond_id);
        let pond_state = object::borrow_mut(pond_obj);
        pond::move_fish_to_for_test(pond_state, fish_id, x, y);
    }

    #[test_only]
    public fun set_food_position_for_test(game_state_obj: &mut Object<GameState>, pond_id: u64, food_id: u64, x: u64, y: u64) {
        let game_state = object::borrow_mut(game_state_obj);
        let pond_obj = table::borrow_mut(&mut game_state.ponds, pond_id);
        let pond_state = object::borrow_mut(pond_obj);
        pond::set_food_position_for_test(pond_state, food_id, x, y);
    }

    #[test_only]
    public fun get_last_food_id(game_state_obj: &Object<GameState>, pond_id: u64): u64 {
        let game_state = object::borrow(game_state_obj);
        let pond_obj = table::borrow(&game_state.ponds, pond_id);
        let pond_state = object::borrow(pond_obj);
        pond::get_last_food_id(pond_state)
    }
}
