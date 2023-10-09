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
    use moveos_std::object;
    use test::m::{Self, TestObject};

    fun main(ctx: &mut Context) {
        let object = m::new_test_object(12);
        let obj = context::new_object<TestObject>(ctx, object);

        let _borrow_object = object::borrow(&obj);
        let (_id, _owner, test_object) = object::unpack(obj);
        m::destroy_test_object(test_object);
    }
}

