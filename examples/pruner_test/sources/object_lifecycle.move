// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module pruner_test::object_lifecycle {
    use std::signer;
    use moveos_std::object::{Self, Object, ObjectID};

    struct TestObject has key, store {
        id: u64,
        value: u64,
    }

    /// Deterministically derive an ObjectID from a seed.
    public fun calc_object_id(seed: u64): ObjectID {
        object::custom_object_id<u64, TestObject>(seed)
    }

    /// Create a simple object for testing
    public entry fun create_object(account: &signer, id: u64, value: u64) {
        let obj = object::new(TestObject { id, value });
        object::transfer(obj, signer::address_of(account));
    }

    /// Create an object with deterministic ID derived from the seed.
    /// If the object already exists, this will abort via object runtime.
    public entry fun create_named(account: &signer, seed: u64, value: u64) {
        let _obj_id = calc_object_id(seed);
        // Use public constructor to avoid friend-only restriction
        let obj = object::new_with_id(seed, TestObject { id: seed, value });
        object::transfer(obj, signer::address_of(account));
    }

    /// Update an existing object
    public entry fun update_object(obj: &mut Object<TestObject>, new_value: u64) {
        let test_obj = object::borrow_mut(obj);
        test_obj.value = new_value;
    }

    /// Update an object using deterministic ID.
    public entry fun update_named(account: &signer, seed: u64, new_value: u64) {
        let obj_id = calc_object_id(seed);
        let obj_ref = object::borrow_mut_object<TestObject>(account, obj_id);
        let test_obj = object::borrow_mut(obj_ref);
        test_obj.value = new_value;
    }

    /// Remove an object
    public entry fun remove_object(obj: Object<TestObject>) {
        let TestObject { id: _, value: _ } = object::remove(obj);
    }

    /// Remove an object using deterministic ID.
    public entry fun remove_named(account: &signer, seed: u64) {
        let obj_id = calc_object_id(seed);
        let obj = object::take_object<TestObject>(account, obj_id);
        let TestObject { id: _, value: _ } = object::remove(obj);
    }
}
