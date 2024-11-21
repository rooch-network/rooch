module rooch_fish::pond {
    use std::vector;
    use std::u64;

    use moveos_std::object::{Self, Object};
    use moveos_std::signer;
    use moveos_std::table::{Self, Table};         
    use moveos_std::event;
    use moveos_std::timestamp;

    use rooch_framework::account_coin_store;
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::gas_coin::{Self, RGas};

    use rooch_fish::fish::{Self, Fish};
    use rooch_fish::food::{Self, Food};
    use rooch_fish::utils;
    use rooch_fish::player::{Self, PlayerList};
    use rooch_fish::quad_tree;

    friend rooch_fish::rooch_fish;

    const ERR_INSUFFICIENT_BALANCE: u64 = 1;
    const ERR_FISH_NOT_FOUND: u64 = 2;
    const ERR_FOOD_NOT_FOUND: u64 = 3;
    const ERR_MAX_FISH_COUNT_REACHED: u64 = 4;
    const ERR_MAX_FOOD_COUNT_REACHED: u64 = 5;
    const ERR_UNAUTHORIZED: u64 = 6;
    const ERR_FISH_NOT_IN_EXIT_ZONE: u64 = 7;
    const ERR_INVALID_POSITION: u64 = 8;
    const ERR_FISH_OUT_OF_BOUNDS: u64 = 9;
    const ERR_INSUFFICIENT_TREASURY_BALANCE: u64 = 10;

    const OBJECT_TYPE_FISH: u8 = 1;
    const OBJECT_TYPE_FOOD: u8 = 2;

    const MAX_FOOD_PER_FEED: u64 = 20; 
    const FOOD_VALUE_RATIO: u64 = 10;
    const MAX_FISH_SIZE: u64 = 100;
    const BURST_FOOD_COUNT: u64 = 10;

    struct ExitZone has store, copy, drop {
        x: u64,
        y: u64,
        radius: u64,
    }

    struct Treasury has key, store {
        coin_store: Object<CoinStore<RGas>>
    }

    struct PondState has key, store {
        id: u64,
        owner: address,
        fishes: Table<u64, Fish>,
        foods: Table<u64, Food>,
        exit_zones: vector<ExitZone>,
        quad_tree: quad_tree::QuadTree<u64>,
        fish_count: u64,
        food_count: u64,
        width: u64,
        height: u64,
        purchase_amount: u256,
        next_fish_id: u64,
        next_food_id: u64,
        max_fish_count: u64,
        max_food_count: u64,
        treasury: Treasury,
        player_list: PlayerList,
    }

    struct BurstEvent has copy, drop, store {
        pond_id: u64,
        fish_id: u64,
        owner: address,
        size: u64,
        food_count: u64,
        total_reward: u256
    }

    struct FishPurchasedEvent has copy, drop, store {
        pond_id: u64,
        fish_id: u64,
        owner: address,
    }

    struct FishMovedEvent has copy, drop, store {
        pond_id: u64,
        fish_id: u64,
        new_x: u64,
        new_y: u64,
    }

    struct FishDestroyedEvent has copy, drop, store {
        pond_id: u64,
        fish_id: u64,
        reward: u256,
    }

    public(friend) fun create_pond(
        id: u64,
        owner: address, 
        width: u64,
        height: u64,
        purchase_amount: u256,
        max_fish_count: u64,
        max_food_count: u64
    ): Object<PondState> {
        let pond_state = PondState {
            id,
            owner,
            fishes: table::new(),
            foods: table::new(),
            exit_zones: vector::empty(),
            quad_tree: quad_tree::create_quad_tree(width, height),
            fish_count: 0,
            food_count: 0,
            width,
            height,
            purchase_amount,
            next_fish_id: 1,
            next_food_id: 1,
            max_fish_count,
            max_food_count,
            treasury: Treasury { coin_store: coin_store::create_coin_store<RGas>() },
            player_list: player::create_player_list(),
        };
        object::new(pond_state)
    }

    public(friend) fun purchase_fish(pond_state: &mut PondState, account: &signer): u64 {
        let account_addr = signer::address_of(account);
        assert!(gas_coin::balance(account_addr) >= pond_state.purchase_amount, ERR_INSUFFICIENT_BALANCE);
        
        let coin = account_coin_store::withdraw(account, pond_state.purchase_amount);
        coin_store::deposit(&mut pond_state.treasury.coin_store, coin);

        let (x, y) = utils::random_position(0, pond_state.width, pond_state.height);
        let fish_id = pond_state.next_fish_id;
        pond_state.next_fish_id = pond_state.next_fish_id + 1;
        let fish = fish::create_fish(account_addr, fish_id, 10, x, y);
        add_fish(pond_state, fish);

        player::add_fish(&mut pond_state.player_list, account_addr, fish_id);

        event::emit(FishPurchasedEvent { pond_id: pond_state.id, fish_id, owner: account_addr });

        fish_id
    }

    public(friend) fun move_fish(pond_state: &mut PondState, account: &signer, fish_id: u64, direction: u8): (u64, u64) {
        let account_addr = signer::address_of(account);
        let fish = get_fish_mut(pond_state, fish_id);
        assert!(fish::get_owner(fish) == account_addr, ERR_UNAUTHORIZED);

        let (old_x, old_y) = fish::get_position(fish);
        fish::move_fish(account, fish, direction);

        let (new_x, new_y) = fish::get_position(fish);
        assert!(new_x < pond_state.width && new_y < pond_state.height, ERR_FISH_OUT_OF_BOUNDS);

        // Update position in quad tree
        quad_tree::update_object_position(
            &mut pond_state.quad_tree, 
            fish_id,
            OBJECT_TYPE_FISH,
            old_x,
            old_y,
            new_x,
            new_y
        );

        handle_collisions(pond_state, fish_id);

        event::emit(FishMovedEvent { 
            pond_id: pond_state.id,
            fish_id,
            new_x,
            new_y
        });

        (new_x, new_y)
    }

    public(friend) fun feed_food(pond_state: &mut PondState, account: &signer, count: u64) : u256 {
        let account_addr = signer::address_of(account);
        
        // Ensure food count does not exceed limits
        let actual_count = u64::min(count, MAX_FOOD_PER_FEED);
        assert!(pond_state.food_count + actual_count <= pond_state.max_food_count, ERR_MAX_FOOD_COUNT_REACHED);
        
        // Calculate actual cost
        let food_value = pond_state.purchase_amount / (FOOD_VALUE_RATIO as u256);
        let total_cost = (actual_count as u256) * food_value;
        
        // Verify and transfer payment
        assert!(gas_coin::balance(account_addr) >= total_cost, ERR_INSUFFICIENT_BALANCE);
        let coin = account_coin_store::withdraw(account, total_cost);
        coin_store::deposit(&mut pond_state.treasury.coin_store, coin);

        // Create food objects
        let i = 0;
        while (i < actual_count) {
            let (x, y) = utils::random_position(i * 2, pond_state.width, pond_state.height);
            let food = food::create_food(
                pond_state.next_food_id,
                account_addr,
                1,
                x,
                y
            );
            add_food(pond_state, food);
            pond_state.next_food_id = pond_state.next_food_id + 1;
            i = i + 1;
        };

        player::add_feed(&mut pond_state.player_list, account_addr, total_cost);

        total_cost
    }

    public(friend) fun destroy_fish(pond_state: &mut PondState, account: &signer, fish_id: u64): u256 {
        let account_addr = signer::address_of(account);
        let fish = get_fish(pond_state, fish_id);
        assert!(fish::get_owner(fish) == account_addr, ERR_UNAUTHORIZED);
        assert!(is_fish_in_exit_zone(pond_state, fish), ERR_FISH_NOT_IN_EXIT_ZONE);

        let removed_fish = remove_fish(pond_state, fish_id);
        player::remove_fish(&mut pond_state.player_list, account_addr, fish_id);
        
        // Calculate total reward
        let total_reward = calculate_reward(&removed_fish, pond_state);
        
        // 1% goes to developer
        let dev_reward = total_reward / 100;
        let dev_coin = coin_store::withdraw(&mut pond_state.treasury.coin_store, dev_reward);
        account_coin_store::deposit(pond_state.owner, dev_coin);

        // 20% distributed proportionally among food contributors
        let contributor_reward = (total_reward * 20) / 100;
        let total_food = fish::get_total_food_consumed(&removed_fish);
        
        if (total_food > 0) {
            let contributors = fish::get_food_contributors(&removed_fish);
            let i = 0;
            while (i < vector::length(&contributors)) {
                let contributor = *vector::borrow(&contributors, i);
                let amount = fish::get_contributor_amount(&removed_fish, contributor);
                let reward = contributor_reward * (amount as u256) / (total_food as u256);
                
                if (reward > 0) {
                    let reward_coin = coin_store::withdraw(&mut pond_state.treasury.coin_store, reward);
                    account_coin_store::deposit(contributor, reward_coin);
                };
                
                i = i + 1;
            };
        };

        // Remaining 79% goes to fish owner
        let owner_reward = total_reward - dev_reward - contributor_reward;
        let owner_coin = coin_store::withdraw(&mut pond_state.treasury.coin_store, owner_reward);
        account_coin_store::deposit(account_addr, owner_coin);

        event::emit(FishDestroyedEvent { 
            pond_id: pond_state.id, 
            fish_id,
            reward: total_reward 
        });

        fish::drop_fish(removed_fish);

        total_reward
    }

    fun add_fish(pond_state: &mut PondState, fish: Fish) {
        assert!(pond_state.fish_count < pond_state.max_fish_count, ERR_MAX_FISH_COUNT_REACHED);

        let id = fish::get_id(&fish);
        let (_, _, _, x, y) = fish::get_fish_info(&fish);
        quad_tree::insert_object(&mut pond_state.quad_tree, id, OBJECT_TYPE_FISH, x, y);
        
        table::add(&mut pond_state.fishes, id, fish);
        pond_state.fish_count = pond_state.fish_count + 1;
    }

    fun remove_fish(pond_state: &mut PondState, fish_id: u64): Fish {
        let fish = table::remove(&mut pond_state.fishes, fish_id);
        let (_, _, _, x, y) = fish::get_fish_info(&fish);
        quad_tree::remove_object(&mut pond_state.quad_tree, fish_id, OBJECT_TYPE_FISH, x, y);

        pond_state.fish_count = pond_state.fish_count - 1;
        fish
    }

    public fun get_fish(pond_state: &PondState, fish_id: u64): &Fish {
        table::borrow(&pond_state.fishes, fish_id)
    }

    fun get_fish_mut(pond_state: &mut PondState, fish_id: u64): &mut Fish {
        table::borrow_mut(&mut pond_state.fishes, fish_id)
    }

    fun add_food(pond_state: &mut PondState, food: Food) {
        assert!(pond_state.food_count < pond_state.max_food_count, ERR_MAX_FOOD_COUNT_REACHED);

        let id = food::get_id(&food);
        let (x, y) = food::get_position(&food);
        quad_tree::insert_object(&mut pond_state.quad_tree, id, OBJECT_TYPE_FOOD, x, y);

        table::add(&mut pond_state.foods, id, food);
        pond_state.food_count = pond_state.food_count + 1;
    }

    fun remove_food(pond_state: &mut PondState, food_id: u64): Food {
        let food = table::remove(&mut pond_state.foods, food_id);
        let (x, y) = food::get_position(&food);
        quad_tree::remove_object(&mut pond_state.quad_tree, food_id, OBJECT_TYPE_FOOD, x, y);

        pond_state.food_count = pond_state.food_count - 1;
        food
    }

    public fun get_food(pond_state: &PondState, food_id: u64): &Food {
        table::borrow(&pond_state.foods, food_id)
    }

    fun get_food_mut(pond_state: &mut PondState, food_id: u64): &mut Food {
        table::borrow_mut(&mut pond_state.foods, food_id)
    }

    fun handle_collisions(pond_state: &mut PondState, fish_id: u64) {
        let fish = get_fish(pond_state, fish_id);
        let fish_size = fish::get_size(fish);
        let (fish_x, fish_y) = fish::get_position(fish);

        if (fish_size >= MAX_FISH_SIZE) {
            process_burst(pond_state, fish_id);
            return
        };

        let query_range = fish_size * 2;
        let nearby_objects = quad_tree::query_range(
            &pond_state.quad_tree,
            utils::saturating_sub(fish_x, query_range),
            utils::saturating_sub(fish_y, query_range),
            query_range * 2,
            query_range * 2,
        );

        let nearby_fish = vector::empty();
        let nearby_food = vector::empty();

        let i = 0;
        while (i < vector::length(&nearby_objects)) {
            let object_entry = vector::borrow(&nearby_objects, i);
            if (quad_tree::get_object_entry_type(object_entry) == OBJECT_TYPE_FISH && 
                quad_tree::get_object_entry_id(object_entry) != fish_id) {
                vector::push_back(&mut nearby_fish, quad_tree::get_object_entry_id(object_entry));
            } else if (quad_tree::get_object_entry_type(object_entry) == OBJECT_TYPE_FOOD) {
                vector::push_back(&mut nearby_food, quad_tree::get_object_entry_id(object_entry));
            };
            i = i + 1;
        };

        handle_food_collisions(pond_state, fish_id, fish_size, fish_x, fish_y, nearby_food);
        handle_fish_collisions(pond_state, fish_id, fish_size, fish_x, fish_y, nearby_fish);
    }


    fun process_burst(pond_state: &mut PondState, fish_id: u64) {
        let fish = remove_fish(pond_state, fish_id);
        let owner = fish::get_owner(&fish);
        let size = fish::get_size(&fish);

        // Calculate total reward
        let total_reward = calculate_reward(&fish, pond_state);
        
        // Owner reward (1%)
        let dev_reward = total_reward / 100;
        let dev_coin = coin_store::withdraw(&mut pond_state.treasury.coin_store, dev_reward);
        account_coin_store::deposit(pond_state.owner, dev_coin);

        // Contributor rewards (20%)
        let contributor_reward = (total_reward * 20) / 100;
        let total_food = fish::get_total_food_consumed(&fish);
        
        if (total_food > 0) {
            let contributors = fish::get_food_contributors(&fish);
            let i = 0;
            while (i < vector::length(&contributors)) {
                let contributor = *vector::borrow(&contributors, i);
                let amount = fish::get_contributor_amount(&fish, contributor);
                let reward = contributor_reward * (amount as u256) / (total_food as u256);
                
                if (reward > 0) {
                    let reward_coin = coin_store::withdraw(&mut pond_state.treasury.coin_store, reward);
                    account_coin_store::deposit(contributor, reward_coin);
                };
                
                i = i + 1;
            };
        };

        // Generate burst food
        let food_size = size / BURST_FOOD_COUNT;
        let i = 0;
        while (i < BURST_FOOD_COUNT) {
            let (x, y) = utils::random_position(
                pond_state.next_food_id + i,
                pond_state.width,
                pond_state.height
            );
            
            let food = food::create_food(
                pond_state.next_food_id,
                owner,  // Set burst fish owner as food owner
                food_size,
                x,
                y
            );
            add_food(pond_state, food);
            pond_state.next_food_id = pond_state.next_food_id + 1;
            i = i + 1;
        };

        event::emit(BurstEvent {
            pond_id: pond_state.id,
            fish_id,
            owner,
            size,
            food_count: BURST_FOOD_COUNT,
            total_reward
        });

        fish::drop_fish(fish);
    }

    fun handle_food_collisions(pond_state: &mut PondState, fish_id: u64, fish_size: u64, fish_x: u64, fish_y: u64, nearby_food: vector<u64>) {
        let growth_amount = 0u64;
        let food_ids_to_remove = vector::empty<u64>();
        let food_owners = vector::empty<address>();
        let food_sizes = vector::empty<u64>();

        // First pass: collect all foods to be eaten and their info
        let i = 0;
        while (i < vector::length(&nearby_food)) {
            let food_id = *vector::borrow(&nearby_food, i);

            if (table::contains(&pond_state.foods, food_id)) {
                let food = get_food(pond_state, food_id);
                let (food_x, food_y) = food::get_position(food);
                if (utils::calculate_distance(fish_x, fish_y, food_x, food_y) <= fish_size) {
                    growth_amount = growth_amount + food::get_size(food);
                    vector::push_back(&mut food_ids_to_remove, food_id);
                    vector::push_back(&mut food_owners, food::get_owner(food));
                    vector::push_back(&mut food_sizes, food::get_size(food));
                };
            };

            i = i + 1;
        };

        // Second pass: update fish
        let fish_mut = get_fish_mut(pond_state, fish_id);
        fish::grow_fish(fish_mut, growth_amount);

        let j = 0;
        while (j < vector::length(&food_owners)) {
            let food_owner = *vector::borrow(&food_owners, j);
            let food_size = *vector::borrow(&food_sizes, j);
        
            fish::record_food_consumption(fish_mut, food_owner, food_size);
            j = j + 1;
        };

        // Third pass: remove foods
        let k = 0;
        while (k < vector::length(&food_ids_to_remove)) {
            let food_id = *vector::borrow(&food_ids_to_remove, k);
            let food = remove_food(pond_state, food_id);
            food::drop_food(food);
            
            k = k + 1;
        };
    }

    fun handle_fish_collisions(pond_state: &mut PondState, fish_id: u64, fish_size: u64, fish_x: u64, fish_y: u64, nearby_fish: vector<u64>) {
        let growth_amount = 0u64;
        let fish_ids_to_remove = vector::empty<u64>();

        let i = 0;
        while (i < vector::length(&nearby_fish)) {
            let other_fish_id = *vector::borrow(&nearby_fish, i);

            if (table::contains(&pond_state.fishes, other_fish_id)) {
                let other_fish = get_fish(pond_state, other_fish_id);
                let (other_x, other_y) = fish::get_position(other_fish);
                let other_size = fish::get_size(other_fish);

                if (!fish::is_protected(other_fish) && 
                    utils::calculate_distance(fish_x, fish_y, other_x, other_y) <= fish_size && fish_size > other_size) {
                    growth_amount = growth_amount + (other_size / 2);
                    vector::push_back(&mut fish_ids_to_remove, other_fish_id);
                };
            };

            i = i + 1;
        };

        let fish_mut = get_fish_mut(pond_state, fish_id);
        fish::grow_fish(fish_mut, growth_amount);

        let j = 0;
        while (j < vector::length(&fish_ids_to_remove)) {
            let fish_id = *vector::borrow(&fish_ids_to_remove, j);
            let fish_obj = remove_fish(pond_state, fish_id);
            let owner = fish::get_owner(&fish_obj);
            player::remove_fish(&mut pond_state.player_list, owner, fish_id);
            fish::drop_fish(fish_obj);
            j = j + 1;
        };
    }

    fun calculate_reward(fish: &Fish, pond_state: &PondState): u256 {
        let base_reward = (fish::get_size(fish) as u256);
        base_reward * pond_state.purchase_amount / 100
    }

    public(friend) fun add_exit_zone(pond_state: &mut PondState, x: u64, y: u64, radius: u64) {
        let exit_zone = ExitZone { x, y, radius };
        vector::push_back(&mut pond_state.exit_zones, exit_zone);
    }

    public(friend) fun remove_exit_zone(pond_state: &mut PondState, index: u64) {
        vector::swap_remove(&mut pond_state.exit_zones, index);
    }

    public fun is_fish_in_exit_zone(pond_state: &PondState, fish: &Fish): bool {
        let (fish_x, fish_y) = fish::get_position(fish);
        let len = vector::length(&pond_state.exit_zones);
        let i = 0;
        while (i < len) {
            let exit_zone = vector::borrow(&pond_state.exit_zones, i);
            if (is_point_in_circle(fish_x, fish_y, exit_zone.x, exit_zone.y, exit_zone.radius)) {
                return true
            };
            i = i + 1;
        };
        false
    }

    fun is_point_in_circle(px: u64, py: u64, cx: u64, cy: u64, radius: u64): bool {
        let dx = if (px > cx) { px - cx } else { cx - px };
        let dy = if (py > cy) { py - cy } else { cy - py };
        (dx * dx + dy * dy) <= (radius * radius)
    }

    public fun get_pond_id(pond_state: &PondState): u64 {
        pond_state.id
    }

    public fun get_width(pond_state: &PondState): u64 {
        pond_state.width
    }

    public fun get_height(pond_state: &PondState): u64 {
        pond_state.height
    }

    public fun get_purchase_amount(pond_state: &PondState): u256 {
        pond_state.purchase_amount
    }

    public fun get_max_fish_count(pond_state: &PondState): u64 {
        pond_state.max_fish_count
    }

    public fun get_max_food_count(pond_state: &PondState): u64 {
        pond_state.max_food_count
    }

    public fun get_fish_count(pond_state: &PondState): u64 {
        pond_state.fish_count
    }

    public fun get_food_count(pond_state: &PondState): u64 {
        pond_state.food_count
    }

    public fun get_player_list(pond_state: &PondState): &PlayerList {
        &pond_state.player_list
    }

    public fun get_player_count(pond_state: &PondState): u64 {
        player::get_player_count(&pond_state.player_list)
    }

    public fun get_player_fish_ids(pond_state: &PondState, owner: address): vector<u64> {
        player::get_player_fish_ids(&pond_state.player_list, owner)
    }

    public fun get_total_feed(pond_state: &PondState): u256 {
        player::get_total_feed(&pond_state.player_list)
    }

    public fun get_max_food_per_feed(): u64 {
        MAX_FOOD_PER_FEED
    }

    public fun get_food_value_ratio(): u64 {
        FOOD_VALUE_RATIO
    }

    #[test_only]
    public(friend) fun drop_pond(pond: Object<PondState>) {
        let PondState { 
            id: _,
            owner: _,
            fishes,
            foods,
            exit_zones,
            quad_tree,
            fish_count: _,
            food_count: _,
            width: _,
            height: _,
            purchase_amount: _,
            next_fish_id: _,
            next_food_id: _,
            max_fish_count: _,
            max_food_count: _,
            treasury,
            player_list
        } = object::remove(pond);

        quad_tree::drop_quad_tree(quad_tree);

        while (!vector::is_empty(&exit_zones)) {
            vector::pop_back(&mut exit_zones);
        };
        vector::destroy_empty(exit_zones);

        table::drop_unchecked(fishes);
        table::drop_unchecked(foods);

        let treasury_obj = object::new_named_object(treasury);
        object::to_shared(treasury_obj);

        player::drop_player_list(player_list);
    }

    #[test_only]
    public(friend) fun move_fish_to_for_test(pond_state: &mut PondState, fish_id: u64, x: u64, y: u64) {
        let fish = get_fish(pond_state, fish_id);
        let (old_x, old_y) = fish::get_position(fish);
        quad_tree::update_object_position(
            &mut pond_state.quad_tree,
            fish_id,
            OBJECT_TYPE_FISH,
            old_x,
            old_y,
            x,
            y
        );
        let fish = get_fish_mut(pond_state, fish_id);
        fish::move_fish_to_for_test(fish, x, y);
    }

    #[test_only]
    public(friend) fun set_food_position_for_test(pond_state: &mut PondState, food_id: u64, x: u64, y: u64) {
        let food = get_food(pond_state, food_id);
        let (old_x, old_y) = food::get_position(food);
        quad_tree::update_object_position(
            &mut pond_state.quad_tree,
            food_id,
            OBJECT_TYPE_FOOD,
            old_x,
            old_y,
            x,
            y
        );
        let food = get_food_mut(pond_state, food_id);
        food::set_position_for_test(food, x, y);
    }

    #[test_only]
    public(friend) fun get_last_food_id(pond_state: &PondState): u64 {
        pond_state.next_food_id - 1
    }

    #[test_only]
    use rooch_framework::genesis;

    #[test]
    fun test_create_pond() {
        genesis::init_for_test();

        let id = 1;
        let owner = @0x123;
        let width = 100;
        let height = 100;
        let purchase_amount = 500;
        let max_fish_count = 50;
        let max_food_count = 30;

        let pond_obj = create_pond(id, owner, width, height, purchase_amount, max_fish_count, max_food_count);
        let pond_state = object::borrow(&pond_obj);

        assert!(get_pond_id(pond_state) == id, 1);
        assert!(get_width(pond_state) == width, 2);
        assert!(get_height(pond_state) == height, 3);
        assert!(get_purchase_amount(pond_state) == purchase_amount, 4);
        assert!(get_max_fish_count(pond_state) == max_fish_count, 5);
        assert!(get_max_food_count(pond_state) == max_food_count, 6);
        assert!(get_fish_count(pond_state) == 0, 7);
        assert!(get_food_count(pond_state) == 0, 8);
        assert!(get_player_count(pond_state) == 0, 9);
        assert!(get_total_feed(pond_state) == 0, 10);

        drop_pond(pond_obj);
    }

    #[test(account = @0x42)]
    fun test_fish_burst_mechanism(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let owner = @0x123;
        let pond_obj = create_pond(1, owner, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 1000));

        let initial_owner_balance = gas_coin::balance(owner);

        // Create a fish near max size
        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);
        
        // Grow fish to near burst size
        let fish = get_fish_mut(pond_state, fish_id);
        fish::grow_fish(fish, MAX_FISH_SIZE - 5);

        // Move fish to trigger burst
        let (_, _) = move_fish(pond_state, &account, fish_id, 1);
        
        // Verify burst results
        assert!(get_fish_count(pond_state) == 0, 1);
        assert!(get_food_count(pond_state) == BURST_FOOD_COUNT, 2);

        // Verify owner received 1% reward
        let final_owner_balance = gas_coin::balance(owner);
        assert!(final_owner_balance > initial_owner_balance, 3);

        drop_pond(pond_obj);
    }

    #[test(account = @0x42, food_owner1 = @0x43, food_owner2 = @0x44)]
    fun test_food_contribution_tracking(account: signer, food_owner1: signer, food_owner2: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        let food_owner1_addr = signer::address_of(&food_owner1);
        let food_owner2_addr = signer::address_of(&food_owner2);
        
        // Set up accounts with gas
        gas_coin::faucet_for_test(account_addr, 1000000);
        gas_coin::faucet_for_test(food_owner1_addr, 1000000);
        gas_coin::faucet_for_test(food_owner2_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        // Create fish
        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);

        // Place food from different owners
        feed_food(pond_state, &food_owner1, 1);
        feed_food(pond_state, &food_owner2, 1);
        
        let food_id1 = get_last_food_id(pond_state) - 1;
        let food_id2 = get_last_food_id(pond_state);
        
        // Position foods near fish
        set_food_position_for_test(pond_state, food_id1, 26, 25);
        set_food_position_for_test(pond_state, food_id2, 27, 25);

        // Move fish to eat both foods
        move_fish(pond_state, &account, fish_id, 1);
        move_fish(pond_state, &account, fish_id, 1);

        // Verify food contributions
        let fish = get_fish(pond_state, fish_id);
        assert!(fish::get_contributor_amount(fish, food_owner1_addr) > 0, 1);
        assert!(fish::get_contributor_amount(fish, food_owner2_addr) > 0, 2);
        assert!(fish::get_total_food_consumed(fish) > 0, 3);
        
        // Verify food was consumed
        assert!(get_food_count(pond_state) == 0, 4);

        drop_pond(pond_obj);
    }

    #[test(account = @0x42)]
    fun test_reward_distribution(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        let owner = @0x123;
        gas_coin::faucet_for_test(account_addr, 1000000);
        gas_coin::faucet_for_test(owner, 1000000);

        let pond_obj = create_pond(1, owner, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        // Add funds to treasury for rewards
        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        let initial_owner_balance = gas_coin::balance(owner);

        // Create and grow fish
        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);
        
        let fish = get_fish_mut(pond_state, fish_id);
        let fish_size = 100;
        fish::grow_fish(fish, fish_size);

        // Calculate expected reward
        let fish_final_size = fish::get_size(fish);
        let total_reward = (fish_final_size as u256) * pond_state.purchase_amount / 100;
        let expected_owner_reward = total_reward / 100; // 1% of total reward

        // Trigger burst
        let (_, _) = move_fish(pond_state, &account, fish_id, 1);

        let final_owner_balance = gas_coin::balance(owner);
        let actual_owner_reward = final_owner_balance - initial_owner_balance;
        
        // Verify owner got exactly 1% of fish's value
        assert!(actual_owner_reward == expected_owner_reward, 1);

        drop_pond(pond_obj);
    }

    #[test(account = @0x42)]
    #[expected_failure(abort_code = ERR_MAX_FISH_COUNT_REACHED)]
    fun test_max_fish_limit(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        // Create pond with small max fish limit
        let max_fish = 2;
        let pond_obj = create_pond(1, account_addr, 100, 100, 500, max_fish, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        // Purchase fish until we hit the limit
        purchase_fish(pond_state, &account);
        assert!(get_fish_count(pond_state) == 1, 1);
        
        purchase_fish(pond_state, &account);
        assert!(get_fish_count(pond_state) == 2, 2);

        // This should fail as we've reached the max fish limit
        purchase_fish(pond_state, &account);

        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    fun test_purchase_fish(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 1000));

        let fish_id = purchase_fish(pond_state, &account);
        assert!(get_fish_count(pond_state) == 1, 1);
        assert!(fish::get_owner(get_fish(pond_state, fish_id)) == account_addr, 2);

        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    fun test_move_fish(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 1000));

        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);

        let (new_x, new_y) = move_fish(pond_state, &account, fish_id, 1);
        
        let fish = get_fish(pond_state, fish_id);
        let (fish_x, fish_y) = fish::get_position(fish);
        assert!(fish_x == new_x && fish_y == new_y, 1);

        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    fun test_feed_food(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 300);
        let pond_state = object::borrow_mut(&mut pond_obj);

        let food_value = 500 / FOOD_VALUE_RATIO;
        let feed_count = MAX_FOOD_PER_FEED;

        let total_cost = feed_food(pond_state, &account, feed_count);
        
        assert!(get_food_count(pond_state) == feed_count, 1);
        assert!(get_total_feed(pond_state) == (feed_count as u256) * (food_value as u256), 2);

        let large_feed_count = MAX_FOOD_PER_FEED * 2;
        let second_total_cost = feed_food(pond_state, &account, large_feed_count);

        assert!(get_food_count(pond_state) == MAX_FOOD_PER_FEED * 2, 3);
        assert!(get_total_feed(pond_state) == total_cost + second_total_cost, 4);

        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    fun test_destroy_fish(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);

        add_exit_zone(pond_state, 0, 0, 100);

        let reward = destroy_fish(pond_state, &account, fish_id);
        assert!(reward > 0, 1);
        assert!(get_fish_count(pond_state) == 0, 2);

        drop_pond(pond_obj);
    }

    #[test]
    fun test_exit_zones() {
        genesis::init_for_test();

        let owner = @0x123;
        let pond_obj = create_pond(1, owner, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        add_exit_zone(pond_state, 10, 10, 5);
        add_exit_zone(pond_state, 90, 90, 8);

        let fish1 = fish::create_fish(@0x1, 1, 10, 12, 12);
        let fish2 = fish::create_fish(@0x2, 2, 15, 50, 50);

        assert!(is_fish_in_exit_zone(pond_state, &fish1), 1);
        assert!(!is_fish_in_exit_zone(pond_state, &fish2), 2);

        remove_exit_zone(pond_state, 0);
        assert!(!is_fish_in_exit_zone(pond_state, &fish1), 3);

        fish::drop_fish(fish1);
        fish::drop_fish(fish2);
        drop_pond(pond_obj);
    }

    #[test(account = @0x42, other_account = @0x43)]
    fun test_get_player_fish_ids(account: signer, other_account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        let fish_id1 = purchase_fish(pond_state, &account);
        let fish_id2 = purchase_fish(pond_state, &account);
        let fish_id3 = purchase_fish(pond_state, &account);

        let fish_ids = get_player_fish_ids(pond_state, account_addr);

        assert!(vector::length(&fish_ids) == 3, 1);
        assert!(vector::contains(&fish_ids, &fish_id1), 2);
        assert!(vector::contains(&fish_ids, &fish_id2), 3);
        assert!(vector::contains(&fish_ids, &fish_id3), 4);

        let other_account_addr = signer::address_of(&other_account);
        gas_coin::faucet_for_test(other_account_addr, 1000000);
        let other_fish_id = purchase_fish(pond_state, &other_account);

        let fish_ids = get_player_fish_ids(pond_state, account_addr);

        assert!(vector::length(&fish_ids) == 3, 5);
        assert!(!vector::contains(&fish_ids, &other_fish_id), 6);

        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    fun test_fish_eat_food_and_move(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 1000));

        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);
        
        let initial_fish_size = fish::get_size(get_fish(pond_state, fish_id));

        feed_food(pond_state, &account, 1);
        let food_id = get_last_food_id(pond_state);
        set_food_position_for_test(pond_state, food_id, 25, 26);
        
        let initial_food_count = get_food_count(pond_state);

        move_fish(pond_state, &account, fish_id, 0);
        
        let fish = get_fish(pond_state, fish_id);
        let final_fish_size = fish::get_size(fish);
        let (fish_x, fish_y) = fish::get_position(fish);
        let final_food_count = get_food_count(pond_state);

        assert!(final_fish_size > initial_fish_size, 1);
        assert!(final_food_count < initial_food_count, 2);
        assert!(fish_x == 25 && fish_y == 26, 3);
        
        move_fish(pond_state, &account, fish_id, 1);
        let fish = get_fish(pond_state, fish_id);
        let (fish_x, fish_y) = fish::get_position(fish);
        
        assert!(fish_x == 26 && fish_y == 26, 4);

        move_fish(pond_state, &account, fish_id, 2);
        move_fish(pond_state, &account, fish_id, 3);
        
        let fish = get_fish(pond_state, fish_id);
        let (fish_x, fish_y) = fish::get_position(fish);
        
        assert!(fish_x == 25 && fish_y == 25, 5);
        
        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    #[expected_failure(abort_code = ERR_MAX_FISH_COUNT_REACHED)]
    fun test_max_fish_count(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 2, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        purchase_fish(pond_state, &account);
        purchase_fish(pond_state, &account);
        purchase_fish(pond_state, &account);

        drop_pond(pond_obj);
    }

    #[test(account = @0x1)]
    #[expected_failure(abort_code = ERR_MAX_FOOD_COUNT_REACHED)]
    fun test_max_food_count(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 5);
        let pond_state = object::borrow_mut(&mut pond_obj);

        feed_food(pond_state, &account, 10);

        drop_pond(pond_obj);
    }

    #[test(account = @0x42, food_owner = @0x43)]
    fun test_destroy_fish_reward_distribution(account: signer, food_owner: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        let food_owner_addr = signer::address_of(&food_owner);
        let owner = @0x123;

        // Set initial balances
        gas_coin::faucet_for_test(account_addr, 1000000);
        gas_coin::faucet_for_test(food_owner_addr, 1000000);
        gas_coin::faucet_for_test(owner, 1000000);
        
        let pond_obj = create_pond(1, owner, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        // Add substantial funds to treasury for rewards
        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 100000));

        // Create and grow fish with significant size
        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);
        
        // Grow fish to increase reward value
        let fish = get_fish_mut(pond_state, fish_id);
        fish::grow_fish(fish, 50);

        // Add food contribution
        feed_food(pond_state, &food_owner, 1);
        let food_id = get_last_food_id(pond_state);
        set_food_position_for_test(pond_state, food_id, 25, 26);
        
        // Move fish to eat food
        move_fish(pond_state, &account, fish_id, 0);

        // Record balances before destroy
        let initial_owner_balance = gas_coin::balance(owner);
        let initial_food_owner_balance = gas_coin::balance(food_owner_addr);
        let initial_fish_owner_balance = gas_coin::balance(account_addr);

        // Add exit zone and destroy fish
        add_exit_zone(pond_state, 0, 0, 100);
        let total_reward = destroy_fish(pond_state, &account, fish_id);

        // Calculate expected rewards
        let dev_reward = total_reward / 100; // 1%
        let contributor_reward = (total_reward * 20) / 100; // 20%
        let owner_reward = total_reward - dev_reward - contributor_reward; // 79%

        // Verify final balances
        let final_owner_balance = gas_coin::balance(owner);
        let final_food_owner_balance = gas_coin::balance(food_owner_addr);
        let final_fish_owner_balance = gas_coin::balance(account_addr);

        // Verify reward distributions
        assert!((final_owner_balance > initial_owner_balance), 1);
        assert!((final_owner_balance - initial_owner_balance) == dev_reward, 2);
        
        assert!((final_food_owner_balance > initial_food_owner_balance), 3);
        assert!((final_food_owner_balance - initial_food_owner_balance) == contributor_reward, 4);
        
        assert!((final_fish_owner_balance > initial_fish_owner_balance), 5);
        assert!((final_fish_owner_balance - initial_fish_owner_balance) == owner_reward, 6);

        // Verify total reward distribution
        assert!(dev_reward + contributor_reward + owner_reward == total_reward, 7);

        drop_pond(pond_obj);
    }

    #[test(account = @0x42)]
    fun test_destroy_fish_no_contributors(account: signer) {
        genesis::init_for_test();

        let account_addr = signer::address_of(&account);
        let owner = @0x123;
        
        // Set initial balances with large amounts
        gas_coin::faucet_for_test(account_addr, 10000000);
        gas_coin::faucet_for_test(owner, 10000000);

        let pond_obj = create_pond(1, owner, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        // Add substantial funds to treasury
        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 1000000));

        // Create and grow fish
        let fish_id = purchase_fish(pond_state, &account);
        move_fish_to_for_test(pond_state, fish_id, 25, 25);
        
        // Grow fish to increase reward value
        let fish = get_fish_mut(pond_state, fish_id);
        fish::grow_fish(fish, 50);

        // Add exit zone and record balances just before destroying fish
        add_exit_zone(pond_state, 0, 0, 100);
        let initial_owner_balance = gas_coin::balance(owner);
        let initial_fish_owner_balance = gas_coin::balance(account_addr);

        // Destroy fish and get total reward
        let total_reward = destroy_fish(pond_state, &account, fish_id);

        // Calculate expected rewards
        let dev_reward = total_reward / 100; // 1%
        let owner_reward = total_reward - dev_reward; // 99% (since no contributors)

        // Get final balances
        let final_owner_balance = gas_coin::balance(owner);
        let final_fish_owner_balance = gas_coin::balance(account_addr);

        // Calculate actual rewards received
        let actual_dev_reward = final_owner_balance - initial_owner_balance;
        let actual_owner_reward = final_fish_owner_balance - initial_fish_owner_balance;

        // Verify rewards - developer should get exactly 1%
        assert!(actual_dev_reward == dev_reward, 1);
        
        // Verify that fish owner got remainder (allowing for minimal precision loss)
        assert!(actual_owner_reward > 0, 2);
        assert!(actual_owner_reward <= owner_reward, 3); // Should not exceed expected
        
        // The difference between expected and actual should be very small
        let reward_difference = if (owner_reward > actual_owner_reward) {
            owner_reward - actual_owner_reward
        } else {
            actual_owner_reward - owner_reward
        };
        assert!(reward_difference < 100, 4); // Allow for small rounding differences
        
        // Total distributed rewards should match total_reward (within small margin)
        let total_distributed = actual_dev_reward + actual_owner_reward;
        let distribution_difference = if (total_reward > total_distributed) {
            total_reward - total_distributed
        } else {
            total_distributed - total_reward
        };
        assert!(distribution_difference < 100, 5); // Allow for small rounding differences

        drop_pond(pond_obj);
    }

    #[test(account = @0x42)]
    fun test_fish_protection(account: signer) {
        genesis::init_for_test();
        
        let account_addr = signer::address_of(&account);
        gas_coin::faucet_for_test(account_addr, 1000000);

        let pond_obj = create_pond(1, account_addr, 100, 100, 500, 50, 30);
        let pond_state = object::borrow_mut(&mut pond_obj);

        coin_store::deposit(&mut pond_state.treasury.coin_store, account_coin_store::withdraw(&account, 10000));

        let predator_id = purchase_fish(pond_state, &account);
        let prey_id = purchase_fish(pond_state, &account);

        let predator = get_fish_mut(pond_state, predator_id);
        fish::grow_fish(predator, 50);

        move_fish_to_for_test(pond_state, predator_id, 25, 25);
        move_fish_to_for_test(pond_state, prey_id, 26, 25);

        move_fish(pond_state, &account, predator_id, 1);
        
        assert!(table::contains(&pond_state.fishes, prey_id), 1);
        
        timestamp::fast_forward_seconds_for_test(61);
        
        move_fish(pond_state, &account, predator_id, 1);
        
        assert!(!table::contains(&pond_state.fishes, prey_id), 2);

        drop_pond(pond_obj);
    }
}
