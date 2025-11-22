// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module pruner_test::object_lifecycle {
    use std::signer;
    use moveos_std::account;
    use moveos_std::object::{Self, Object};

    struct TestObject has key, store {
        id: u64,
        value: u64,
    }

    /// Create a simple object for testing
    public entry fun create_object(account: &signer, id: u64, value: u64) {
        let obj = object::new(TestObject { id, value });
        object::transfer(obj, signer::address_of(account));
    }

    /// Update an existing object
    public entry fun update_object(obj: &mut Object<TestObject>, new_value: u64) {
        let test_obj = object::borrow_mut(obj);
        test_obj.value = new_value;
    }

    /// Remove an object
    public entry fun remove_object(obj: Object<TestObject>) {
        let TestObject { id: _, value: _ } = object::remove(obj);
    }
}