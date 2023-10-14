//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    struct TestObject has key{
        f: u8
    }

    public fun new_test_object(f: u8): TestObject {
        TestObject{
            f,
        }
    }

    public fun destroy_test_object(test_object: TestObject) {
        let TestObject{f : _f} = test_object;
    }
}

//check private_generics verify
//# run --signers A
script {
    use moveos_std::context::{Self, Context};
    use moveos_std::object_ref;
    use test::m::{Self, TestObject};

    fun main(ctx: &mut Context) {
        let object = m::new_test_object(12);
        let obj_ref = context::new_object<TestObject>(ctx, object);
        let (_id, _owner, test_object) = context::remove_object<TestObject>(ctx, object_ref::id(&obj_ref));
        m::destroy_test_object(test_object);
    }
}

