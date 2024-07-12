//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    
    use moveos_std::object::{Self, Object};

    struct TestStruct has key, store{
        value: u64
    }

    struct TestContainer has key, store {
        inner_obj: Object<TestStruct>,
    }

    fun init(){
        let obj1 = object::new_named_object(TestStruct{value: 1});
        std::debug::print(&obj1);
        let obj2 = object::new_named_object(TestContainer{inner_obj: obj1});
        std::debug::print(&obj2);
        object::transfer(obj2, moveos_std::tx_context::sender());
    }

    public fun unpack_and_tranfer(){
        let obj = object::take_object_extend<TestContainer>(object::named_object_id<TestContainer>());
        let TestContainer{inner_obj} = object::remove(obj);
        let inner_obj_id = object::id(&inner_obj);
        object::transfer(inner_obj, moveos_std::tx_context::sender());
        //borrow the obj again
        let _obj_mut = object::borrow_mut_object_extend<TestStruct>(inner_obj_id);
    }

}

//# run --signers test
script {
    use test::m;

    fun main() {
        m::unpack_and_tranfer();
    }
}