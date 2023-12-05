module quick_start_object_counter::quick_start_object_counter {
    use std::debug;
    use std::signer;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::context::{Self, Context};

    struct ObjectCounter has key, store {
        count_value: u64
    }

    fun create_(ctx: &mut Context, owner: &signer): ObjectID {
        let counter = ObjectCounter { count_value: 0 };
        let owner_addr = signer::address_of(owner);
        let counter_obj = context::new_object(ctx, counter);
        let counter_obj_id = object::id(&counter_obj);
        object::transfer(counter_obj, owner_addr);
        counter_obj_id
    }

    fun init(ctx: &mut Context, owner: &signer) {
        let counter_obj_id = create_(ctx, owner);
        debug::print(&counter_obj_id);
    }

    fun increase_(counter_obj: &mut Object<ObjectCounter>) {
        let counter = object::borrow_mut(counter_obj);
        counter.count_value = counter.count_value + 1;
    }

    public entry fun increase(counter_obj: &mut Object<ObjectCounter>) {
        increase_(counter_obj);
    }
}