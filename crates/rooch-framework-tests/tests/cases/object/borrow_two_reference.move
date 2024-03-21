//# init --addresses test=0x42

//# publish

module test::m {
    
    use moveos_std::object;

    struct TestStruct has key, store{
        value: u64
    }

    fun init(){
        let obj = object::new_named_object(TestStruct{value: 0});
        std::debug::print(&obj);
        object::transfer(obj, moveos_std::tx_context::sender());
    }

    public fun set_value(test: &mut TestStruct, value: u64){
        test.value = value;
    }

    public fun get_value(test: &TestStruct) : u64{
        test.value
    }
}

// test two ref: will fail
//# run --signers test --args object_id:0xdbac1380a14940361115d51f5d89871c502556428d4eed8d44cd66abd5e0700c
script {
    use moveos_std::object;
    use moveos_std::object::ObjectID;
    use test::m::{Self, TestStruct};

    fun main(obj_id: ObjectID) {
        let object1 = object::borrow_object<TestStruct>(obj_id);
        let object2 = object::borrow_object<TestStruct>(obj_id);
        m::get_value(object::borrow(object1));
        m::get_value(object::borrow(object2));
    }
}

// test two ref in different scope: should be allowed
//# run --signers test --args object_id:0xdbac1380a14940361115d51f5d89871c502556428d4eed8d44cd66abd5e0700c
script {
    use moveos_std::object;
    use moveos_std::object::ObjectID;
    use test::m::{Self, TestStruct};

    fun main(obj_id: ObjectID) {
        {
            let object1 = object::borrow_object<TestStruct>(obj_id);
            m::get_value(object::borrow(object1));
        };
        {
            let object2 = object::borrow_object<TestStruct>(obj_id);
            m::get_value(object::borrow(object2));
        };
    }
}

// test two mut ref : will fail
//# run --signers test --args object_id:0xdbac1380a14940361115d51f5d89871c502556428d4eed8d44cd66abd5e0700c
script {
    use moveos_std::object;
    use moveos_std::object::ObjectID;
    use test::m::{Self, TestStruct};

    fun main(sender: &signer, obj_id: ObjectID) {
        let object1 = object::borrow_mut_object<TestStruct>(sender, obj_id);
        let object2 = object::borrow_mut_object<TestStruct>(sender, obj_id);
        m::set_value(object::borrow_mut(object1), 42);
        m::set_value(object::borrow_mut(object2), 43);
    }
}