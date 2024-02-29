//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    use moveos_std::context::{Self, Context};
    use moveos_std::object;

    struct TestStruct has key, store{
        value: u64
    }

    fun init(ctx: &mut Context){
        let id = context::new_named_object_uid<TestStruct>(ctx);
        let obj = object::new(id, TestStruct{value: 0});
        std::debug::print(&obj);
        object::transfer(obj, context::sender(ctx));
    }

    public fun set_value(test: &mut TestStruct, value: u64){
        test.value = value;
    }

    public fun get_value(test: &TestStruct) : u64{
        test.value
    }
}

// test mut ref and ref both
//# run --signers test --args @0xdbac1380a14940361115d51f5d89871c502556428d4eed8d44cd66abd5e0700c
script {
    use moveos_std::object::{Self, Object};
    use test::m::{Self, TestStruct};

    fun main(obj_from_arg: &mut Object<TestStruct>) {
        let value_from_arg = m::get_value(object::borrow(obj_from_arg));
        let object_from_ctx = object::borrow_object(object::id(obj_from_arg));
        let value_from_ctx = m::get_value(object::borrow(object_from_ctx));
        assert!(value_from_arg == value_from_ctx, 1);
        m::set_value(object::borrow_mut(obj_from_arg), 42);
        let value_from_arg = m::get_value(object::borrow(obj_from_arg));
        let value_from_ctx = m::get_value(object::borrow(object_from_ctx));
        assert!(value_from_arg == 42, 2);
        assert!(value_from_arg == value_from_ctx, 3);
    }
}

// test two mut ref
//# run --signers test --args @0xdbac1380a14940361115d51f5d89871c502556428d4eed8d44cd66abd5e0700c
script {
    use moveos_std::object::{Self, Object};
    use test::m::{Self, TestStruct};

    fun main(sender: &signer, obj_from_arg: &mut Object<TestStruct>) {
        let value_from_arg = m::get_value(object::borrow(obj_from_arg));
        let object_from_ctx = object::borrow_mut_object(sender, object::id(obj_from_arg));
        let value_from_ctx = m::get_value(object::borrow(object_from_ctx));
        assert!(value_from_arg == value_from_ctx, 1);
        
        m::set_value(object::borrow_mut(obj_from_arg), 42);
        m::set_value(object::borrow_mut(object_from_ctx), 420);
        
        let value_from_arg = m::get_value(object::borrow(obj_from_arg));
        let value_from_ctx = m::get_value(object::borrow(object_from_ctx));
        assert!(value_from_ctx == 420, 2);
        assert!(value_from_arg == value_from_ctx, 3);
    }
}