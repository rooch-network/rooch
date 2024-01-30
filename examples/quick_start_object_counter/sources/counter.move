module quick_start_object_counter::quick_start_object_counter {
    use std::signer;
    use moveos_std::object_id::ObjectID;
    use moveos_std::event;
    use moveos_std::object::{Self, Object};
    use moveos_std::context::{Self, Context};

    struct Counter has key, store {
        count_value: u64
    }

    struct UserCounterCreatedEvent has drop {
        id: ObjectID
    }

    fun init(ctx: &mut Context, owner: &signer) {
        create_shared(ctx);
        create_user(ctx, owner);
    }

    fun create_shared(ctx: &mut Context) {
        let counter = Counter { count_value: 0 };
        let counter_obj = context::new_named_object(ctx, counter);
        object::to_shared(counter_obj);
    }

    fun create_user(ctx: &mut Context, owner: &signer): ObjectID {
        let counter = Counter { count_value: 123 };
        let owner_addr = signer::address_of(owner);
        let counter_obj = context::new_object(ctx, counter);
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