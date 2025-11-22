// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module pruner_test::object_lifecycle {
    use std::signer;
    use std::vector;
    use moveos_std::account;
    use moveos_std::object::{Self, Object};
    use moveos_std::table::{Self, Table};

    const EOBJECT_MISSING: u64 = 1;

    struct Registry has key {
        next_id: u64,
        objects: Table<u64, Object<TestObject>>,
    }

    struct TestObject has key, store {
        value: u64,
        data: vector<u8>,
    }

    /// Create a new object of `size` bytes and store it under the caller's registry.
    /// The object is kept in-module so tests can update or remove it by index.
    public entry fun create_object(account: &signer, value: u64, size: u64) {
        let registry = ensure_registry(account);

        let mut data = vector::empty<u8>();
        let mut i = 0;
        while (i < size) {
            vector::push_back(&mut data, (i % 256) as u8);
            i = i + 1;
        };

        let obj = object::new(TestObject { value, data });
        table::add(&mut registry.objects, registry.next_id, obj);
        registry.next_id = registry.next_id + 1;
    }

    /// Update an existing object to create additional versions for pruning.
    public entry fun update_object(account: &signer, idx: u64, new_value: u64) {
        let registry = ensure_registry(account);
        let obj = table::borrow_mut(&mut registry.objects, idx);
        let inner = object::borrow_mut(obj);
        inner.value = new_value;
    }

    /// Remove an object by index, producing stale nodes.
    public entry fun remove_object(account: &signer, idx: u64) {
        let registry = ensure_registry(account);
        if (!table::contains(&registry.objects, idx)) {
            abort EOBJECT_MISSING
        };
        let obj = table::remove(&mut registry.objects, idx);
        let TestObject { value: _, data: _ } = object::remove(obj);
    }

    fun ensure_registry(account: &signer): &mut Registry {
        let sender = signer::address_of(account);
        if (!account::exists_resource<Registry>(sender)) {
            account::move_resource_to(
                account,
                Registry {
                    next_id: 0,
                    objects: table::new<u64, Object<TestObject>>(),
                },
            );
        };
        account::borrow_mut_resource<Registry>(sender)
    }
}
