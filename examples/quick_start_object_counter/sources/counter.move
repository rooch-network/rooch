module quick_start_object_counter::quick_start_object_counter {
    use std::signer;
    use moveos_std::context::{Self, Context};

    struct ObjectCounter has key, store {
        count_value: u64
    }

    fun init(ctx: &mut Context, owner: &signer) {
        create_counter(ctx, owner);
    }

    public fun create_counter(ctx: &mut Context, owner: &signer) {
        let counter = ObjectCounter { count_value: 0 };
        context::move_resource_to(ctx, owner, counter);
    }

    public entry fun increase(ctx: &mut Context, owner: &signer) {
        let owner_addr = signer::address_of(owner);
        let counter = context::borrow_mut_resource<ObjectCounter>(ctx, owner_addr);
        counter.count_value = counter.count_value + 1;
    }

    public fun query(ctx: &Context, owner_addr: address): &u64 {
        // let owner_addr = signer::address_of(owner);
        let counter = context::borrow_resource<ObjectCounter>(ctx, owner_addr);
        &counter.count_value
    }
}