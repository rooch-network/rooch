module quick_start_object_counter::quick_start_object_counter {
    use std::debug;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::signer::address_of;
    use moveos_std::context;
    use moveos_std::context::{Context, new_object, borrow_mut_object};

    struct ObjectCounter has key, store {
        count_value: u64
    }


    fun init(ctx: &mut Context, account: &signer) {
        let counter = ObjectCounter { count_value: 0 };
        // context::move_resource_to(ctx, account, counter);
        let send_addr = address_of(account);
        let obj = new_object(ctx, counter);
        let obj_id = object::id(&obj);
        object::transfer(obj, send_addr);
        debug::print(&obj_id);
    }

    entry fun increase(ctx: &mut Context, account: &signer) {
        let obj_id =  object::id<ObjectCounter>(&Object<ObjectCounter>);
        let obj = context::borrow_mut_object<ObjectCounter>(ctx, account, obj_id);
        let v = object::borrow_mut(obj);

        v.count_value = v.count_value + 1
    }
}