// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_fish::fish {
    use moveos_std::signer;
    use moveos_std::timestamp;
    use moveos_std::simple_map::{Self, SimpleMap};
    use std::vector;

    friend rooch_fish::rooch_fish;
    friend rooch_fish::pond;

    const ErrorNotOwner: u64 = 1;
    const ErrorInvalidDirection: u64 = 2;
    const ErrorUnauthorizedCreation: u64 = 3;

    struct Fish has key, store {
        id: u64,
        owner: address,
        size: u64,
        x: u64,
        y: u64,
        food_contributors: SimpleMap<address, u64>,
        total_food_consumed: u64,
        created_at: u64,
    }

    public(friend) fun create_fish(owner: address, id: u64, size: u64, x: u64, y: u64): Fish {
        Fish {
            id,
            owner,
            size,
            x,
            y,
            food_contributors: simple_map::new(),
            total_food_consumed: 0,
            created_at: timestamp::now_milliseconds(),
        }
    }

    public(friend) fun move_fish(account: &signer, fish: &mut Fish, direction: u8) {
        assert!(signer::address_of(account) == fish.owner, ErrorNotOwner);
        assert!(direction <= 3, ErrorInvalidDirection);

        if (direction == 0) {
            fish.y = fish.y + 1;
        } else if (direction == 1) {
            fish.x = fish.x + 1;
        } else if (direction == 2) {
            fish.y = fish.y - 1;
        } else {
            fish.x = fish.x - 1;
        }
    }

    #[test_only]
    public(friend) fun move_fish_to_for_test(fish: &mut Fish, x: u64, y: u64) {
        fish.x = x;
        fish.y = y;
    }

    public(friend) fun grow_fish(fish: &mut Fish, amount: u64) {
        fish.size = fish.size + amount;
    }

    public(friend) fun record_food_consumption(fish: &mut Fish, food_owner: address, food_size: u64) {
        let current_amount = if (simple_map::contains_key(&fish.food_contributors, &food_owner)) {
            *simple_map::borrow(&fish.food_contributors, &food_owner)
        } else {
            0
        };
        
        let new_amount = current_amount + food_size;
        if (current_amount == 0) {
            simple_map::add(&mut fish.food_contributors, food_owner, new_amount);
        } else {
            *simple_map::borrow_mut(&mut fish.food_contributors, &food_owner) = new_amount;
        };
        
        fish.total_food_consumed = fish.total_food_consumed + food_size;
    }

    public fun get_food_contributors(fish: &Fish): vector<address> {
        simple_map::keys(&fish.food_contributors)
    }

    public fun get_contributor_amount(fish: &Fish, owner: address): u64 {
        if (simple_map::contains_key(&fish.food_contributors, &owner)) {
            *simple_map::borrow(&fish.food_contributors, &owner)
        } else {
            0
        }
    }

    public fun get_total_food_consumed(fish: &Fish): u64 {
        fish.total_food_consumed
    }

    public fun is_protected(fish: &Fish): bool {
        let protection_period = 60000; // 1 minute in milliseconds
        let current_time = timestamp::now_milliseconds();
        current_time - fish.created_at < protection_period
    }

    public fun get_created_at(fish: &Fish): u64 {
        fish.created_at
    }

    public(friend) fun drop_fish(fish: Fish) {
        let Fish { 
            id: _,
            owner: _,
            size: _,
            x: _,
            y: _,
            food_contributors: _,
            total_food_consumed: _,
            created_at: _,
        } = fish;
    }

    public fun get_fish_info(fish: &Fish): (u64, address, u64, u64, u64) {
        (fish.id, fish.owner, fish.size, fish.x, fish.y)
    }

    public fun get_id(fish: &Fish): u64 {
        fish.id
    }

    public fun get_owner(fish: &Fish): address {
        fish.owner
    }

    public fun get_size(fish: &Fish): u64 {
        fish.size
    }

    public fun get_x(fish: &Fish): u64 {
        fish.x
    }

    public fun get_y(fish: &Fish): u64 {
        fish.y
    }

    public fun get_position(fish: &Fish): (u64, u64) {
        (fish.x, fish.y)
    }

    #[test]
    fun test_create_fish() {
        let owner = @0x1;
        let fish = create_fish(owner, 1, 10, 5, 5);
        let (id, fish_owner, size, x, y) = get_fish_info(&fish);

        assert!(id == 1, 1);
        assert!(fish_owner == owner, 2);
        assert!(size == 10, 3);
        assert!(x == 5, 4);
        assert!(y == 5, 5);
        assert!(get_total_food_consumed(&fish) == 0, 6);
        assert!(vector::length(&get_food_contributors(&fish)) == 0, 7);

        drop_fish(fish);
    }

    #[test(owner = @0x42)]
    fun test_move_fish(owner: signer) {
        let owner_addr = signer::address_of(&owner);
        let fish = create_fish(owner_addr, 1, 10, 5, 5);
        
        // Test all directions
        move_fish(&owner, &mut fish, 0); // up
        assert!(get_y(&fish) == 6, 1);
        
        move_fish(&owner, &mut fish, 1); // right
        assert!(get_x(&fish) == 6, 2);
        
        move_fish(&owner, &mut fish, 2); // down
        assert!(get_y(&fish) == 5, 3);
        
        move_fish(&owner, &mut fish, 3); // left
        assert!(get_x(&fish) == 5, 4);

        drop_fish(fish);
    }

    #[test(owner = @0x42)]
    fun test_food_contribution(owner: signer) {
        let owner_addr = signer::address_of(&owner);
        let fish = create_fish(owner_addr, 1, 10, 5, 5);
        
        // Test single contribution
        let food_owner = @0x101;
        record_food_consumption(&mut fish, food_owner, 5);
        assert!(get_contributor_amount(&fish, food_owner) == 5, 1);
        assert!(get_total_food_consumed(&fish) == 5, 2);

        // Test multiple contributions from same owner
        record_food_consumption(&mut fish, food_owner, 3);
        assert!(get_contributor_amount(&fish, food_owner) == 8, 3);
        assert!(get_total_food_consumed(&fish) == 8, 4);

        drop_fish(fish);
    }

    #[test(owner = @0x42)]
    fun test_multiple_contributors(owner: signer) {
        let owner_addr = signer::address_of(&owner);
        let fish = create_fish(owner_addr, 1, 10, 5, 5);
        
        let food_owner1 = @0x101;
        let food_owner2 = @0x102;
        let food_owner3 = @0x103;
        
        record_food_consumption(&mut fish, food_owner1, 5);
        record_food_consumption(&mut fish, food_owner2, 10);
        record_food_consumption(&mut fish, food_owner3, 15);
        
        assert!(get_total_food_consumed(&fish) == 30, 1);
        assert!(get_contributor_amount(&fish, food_owner1) == 5, 2);
        assert!(get_contributor_amount(&fish, food_owner2) == 10, 3);
        assert!(get_contributor_amount(&fish, food_owner3) == 15, 4);
        
        let contributors = get_food_contributors(&fish);
        assert!(vector::length(&contributors) == 3, 5);
        assert!(vector::contains(&contributors, &food_owner1), 6);
        assert!(vector::contains(&contributors, &food_owner2), 7);
        assert!(vector::contains(&contributors, &food_owner3), 8);

        drop_fish(fish);
    }

    #[test(owner = @0x42)]
    fun test_grow_fish(owner: signer) {
        let owner_addr = signer::address_of(&owner);
        let fish = create_fish(owner_addr, 1, 10, 5, 5);
        
        grow_fish(&mut fish, 5);
        assert!(get_size(&fish) == 15, 1);
        
        grow_fish(&mut fish, 10);
        assert!(get_size(&fish) == 25, 2);

        drop_fish(fish);
    }

    #[test(owner = @0x42, non_owner = @0x43)]
    #[expected_failure(abort_code = ErrorNotOwner)]
    fun test_move_fish_non_owner(owner: signer, non_owner: signer) {
        let owner_addr = signer::address_of(&owner);
        let fish = create_fish(owner_addr, 1, 10, 5, 5);
        move_fish(&non_owner, &mut fish, 0);
        drop_fish(fish);
    }

    #[test(owner = @0x42)]
    #[expected_failure(abort_code = ErrorInvalidDirection)]
    fun test_move_fish_invalid_direction(owner: signer) {
        let owner_addr = signer::address_of(&owner);
        let fish = create_fish(owner_addr, 1, 10, 5, 5);
        move_fish(&owner, &mut fish, 4);
        drop_fish(fish);
    }

    #[test]
    fun test_non_contributor() {
        let fish = create_fish(@0x1, 1, 10, 5, 5);
        let non_contributor = @0x999;
        
        assert!(get_contributor_amount(&fish, non_contributor) == 0, 1);
        assert!(!simple_map::contains_key(&fish.food_contributors, &non_contributor), 2);
        
        let contributors = get_food_contributors(&fish);
        assert!(vector::length(&contributors) == 0, 3);
        assert!(!vector::contains(&contributors, &non_contributor), 4);

        drop_fish(fish);
    }

    #[test]
    fun test_position_getters() {
        let fish = create_fish(@0x1, 1, 10, 5, 5);
        
        assert!(get_x(&fish) == 5, 1);
        assert!(get_y(&fish) == 5, 2);
        
        let (x, y) = get_position(&fish);
        assert!(x == 5, 3);
        assert!(y == 5, 4);
        
        move_fish_to_for_test(&mut fish, 10, 15);
        assert!(get_x(&fish) == 10, 5);
        assert!(get_y(&fish) == 15, 6);

        drop_fish(fish);
    }
}
