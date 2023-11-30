module quick_start_object_counter::quick_start_object_counter {
    use std::debug;
    use std::signer;
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::context::{Self, Context};

    struct ObjectCounter has key, store {
        count_value: u64
    }

    fun init(ctx: &mut Context, owner: &signer) {
        create_counter(ctx, owner);
    }

    public fun create_counter(ctx: &mut Context, owner: &signer): ObjectID {
        let counter = ObjectCounter { count_value: 0 };
        let owner_addr = signer::address_of(owner);
        let obj = context::new_object(ctx, counter);
        let id = object::id(&obj);
        debug::print(&id);
        object::transfer(obj, owner_addr);
        id
    }

    entry public fun increase(ctx: &mut Context, owner: &signer) {
        let obj_id = create_counter(ctx, owner);
        let obj = context::borrow_mut_object<ObjectCounter>(ctx, owner, obj_id);
        let v = object::borrow_mut(obj);

        v.count_value = v.count_value + 1
    }
}