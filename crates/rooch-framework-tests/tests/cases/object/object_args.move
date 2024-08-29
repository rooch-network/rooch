//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    
    use moveos_std::object;

    struct TestStruct has key, store{
        value: u64
    }

    fun init(){
        let obj1 = object::new_named_object(TestStruct{value: 1});
        std::debug::print(&obj1);
        object::transfer(obj1, moveos_std::tx_context::sender());
    }

    public fun set_value(test: &mut TestStruct, value: u64){
        test.value = value;
    }

    public fun get_value(test: &TestStruct) : u64{
        test.value
    }
}

// test one ref: expect success
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Object};
    use test::m::{TestStruct};

    fun main(_obj_from_arg: &Object<TestStruct>) {
    }
}

// test one mut ref: expect success
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Object};
    use test::m::{TestStruct};

    fun main(_obj_from_arg: &mut Object<TestStruct>) {
    }
}

// test one value: expect success
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Self, Object};
    use test::m::{TestStruct};

    fun main(obj_from_arg: Object<TestStruct>) {
        object::transfer(obj_from_arg, moveos_std::tx_context::sender());
    }
}

// test one value with type args: expect success
//# run --signers test --type-args 0x42::m::TestStruct --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Self, Object};

    fun main<T:key+store>(obj_from_arg: Object<T>) {
        object::transfer(obj_from_arg, moveos_std::tx_context::sender());
    }
}

// test mut ref and ref both: expect failure
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Self, Object};
    use test::m::{TestStruct};

    fun main(obj_from_arg: &mut Object<TestStruct>) {
        let _obj_from_store = object::borrow_object<TestStruct>(object::id(obj_from_arg));
    }
}

// test two mut ref: expect failure
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Self, Object};
    use test::m::{TestStruct};

    fun main(sender: &signer, obj_from_arg: &mut Object<TestStruct>) {
        let _obj_from_store = object::borrow_mut_object<TestStruct>(sender, object::id(obj_from_arg));
    }
}


// test two repeat mut ref: expect failure
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Object};
    use test::m::{TestStruct};

    fun main(_obj_from_arg: &mut Object<TestStruct>, _obj_from_arg2: &mut Object<TestStruct>) {
    }
}

// test one value and one ref with same object: expect failure
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Self, Object};
    use test::m::{TestStruct};

    fun main(obj_from_arg: Object<TestStruct>, _obj_from_arg2: &Object<TestStruct>) {
        object::transfer(obj_from_arg, moveos_std::tx_context::sender());
    }
}

// test one value and one mut ref with same object: expect failure
//# run --signers test --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc --args object_id:0x1688417d0dfc3020107e88752aa699fbe5709f0b7f91e366c97cb296675901fc
script {
    use moveos_std::object::{Self, Object};
    use test::m::{TestStruct};

    fun main(obj_from_arg: Object<TestStruct>, _obj_from_arg2: &mut Object<TestStruct>) {
        object::transfer(obj_from_arg, moveos_std::tx_context::sender());
    }
}