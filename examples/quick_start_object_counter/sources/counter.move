// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module quick_start_object_counter::quick_start_object_counter {
    use std::signer;
    use moveos_std::event;
    use moveos_std::object::{Self, Object, ObjectID};

    struct Counter has key, store {
        count_value: u64
    }

    struct UserCounterCreatedEvent has drop {
        id: ObjectID
    }

    fun init(owner: &signer) {
        create_shared();
        create_user(owner);
    }

    fun create_shared() {
        let counter = Counter { count_value: 0 };
        let counter_obj = object::new_named_object(counter);
        object::to_shared(counter_obj);
    }

    fun create_user(owner: &signer): ObjectID {
        let counter = Counter { count_value: 123 };
        let owner_addr = signer::address_of(owner);
        let counter_obj = object::new(counter);
        let counter_obj_id = object::id(&counter_obj);
        object::transfer(counter_obj, owner_addr);
        let user_counter_created_event = UserCounterCreatedEvent { id: counter_obj_id };
        event::emit(user_counter_created_event);
        counter_obj_id
    }

    public entry fun increase(counter_obj: &mut Object<Counter>) {
        let counter = object::borrow_mut(counter_obj);
        counter.count_value = counter.count_value + 1;
    }
}
