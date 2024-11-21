// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_fish::food {
    use std::vector;

    friend rooch_fish::rooch_fish;
    friend rooch_fish::pond;

    struct Food has key, store {
        id: u64,
        owner: address,
        size: u64,
        x: u64,
        y: u64,
    }

    public(friend) fun create_food(id: u64, owner: address, size: u64, x: u64, y: u64): Food {
        Food {
            id,
            owner,
            size,
            x,
            y,
        }
    }

    public(friend) fun create_multiple_food(
        ids: vector<u64>,
        owner: address,
        sizes: vector<u64>, 
        xs: vector<u64>, 
        ys: vector<u64>
    ): vector<Food> {
        let len = vector::length(&ids);
        assert!(len == vector::length(&sizes) && len == vector::length(&xs) && len == vector::length(&ys), 0);
        
        let foods = vector::empty();
        let i = 0;
        while (i < len) {
            let food = create_food(
                *vector::borrow(&ids, i),
                owner,
                *vector::borrow(&sizes, i),
                *vector::borrow(&xs, i),
                *vector::borrow(&ys, i)
            );
            vector::push_back(&mut foods, food);
            i = i + 1;
        };
        foods
    }

    public fun get_id(food: &Food): u64 {
        food.id
    }

    public fun get_owner(food: &Food): address {
        food.owner
    }

    public fun get_size(food: &Food): u64 {
        food.size
    }

    public fun get_position(food: &Food): (u64, u64) {
        (food.x, food.y)
    }

    #[test_only]
    public fun set_position_for_test(food: &mut Food, x: u64, y: u64) {
        food.x = x;
        food.y = y;
    }

    public(friend) fun drop_food(food: Food) {
        let Food { id: _, owner: _, size: _, x: _, y: _ } = food;
    }

    #[test_only]
    use moveos_std::signer;

    #[test]
    fun test_create_food() {
        let id = 1;
        let owner = @0x42;
        let size = 5;
        let x = 10;
        let y = 20;

        let food = create_food(id, owner, size, x, y);
        
        assert!(get_id(&food) == id, 1);
        assert!(get_owner(&food) == owner, 2);
        assert!(get_size(&food) == size, 3);
        
        let (food_x, food_y) = get_position(&food);
        assert!(food_x == x, 4);
        assert!(food_y == y, 5);

        drop_food(food);
    }

    #[test]
    fun test_create_multiple_food() {
        let ids = vector[1, 2, 3];
        let owner = @0x42;
        let sizes = vector[5, 10, 15];
        let xs = vector[10, 20, 30];
        let ys = vector[40, 50, 60];

        let foods = create_multiple_food(ids, owner, sizes, xs, ys);
        assert!(vector::length(&foods) == 3, 1);

        let food = vector::pop_back(&mut foods);
        assert!(get_id(&food) == 3, 2);
        assert!(get_owner(&food) == owner, 3);
        assert!(get_size(&food) == 15, 4);
        let (x, y) = get_position(&food);
        assert!(x == 30 && y == 60, 5);

        drop_food(food);
        vector::for_each(foods, |food| drop_food(food));
    }

    #[test(account = @0x42)]
    fun test_food_ownership(account: signer) {
        let owner = signer::address_of(&account);
        let food = create_food(1, owner, 5, 10, 20);
        
        assert!(get_owner(&food) == owner, 1);
        
        drop_food(food);
    }

    #[test]
    fun test_position_setters() {
        let food = create_food(1, @0x42, 5, 10, 20);
        
        set_position_for_test(&mut food, 30, 40);
        let (x, y) = get_position(&food);
        assert!(x == 30 && y == 40, 1);
        
        drop_food(food);
    }

    #[test]
    fun test_multiple_food_validation() {
        let ids = vector[1, 2];
        let sizes = vector[5, 10, 15]; // Mismatched length
        let xs = vector[10, 20];
        let ys = vector[40, 50];

        assert!(
            vector::length(&ids) != vector::length(&sizes) ||
            vector::length(&xs) != vector::length(&ys) ||
            vector::length(&ids) != vector::length(&xs),
            0
        );
    }
}

