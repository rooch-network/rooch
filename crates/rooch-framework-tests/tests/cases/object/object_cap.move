//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    struct TestStruct has key{
        f: u8
    }

    public fun new_test_struct(f: u8): TestStruct {
        TestStruct{
            f,
        }
    }

    public fun destroy_test_struct(test_struct: TestStruct) {
        let TestStruct{f : _f} = test_struct;
    }
}

//check private_generics verify
//# run --signers A
script {
    use moveos_std::context::{Self, Context};
    use moveos_std::object;
    use test::m::{Self, TestStruct};

    fun main(ctx: &mut Context) {
        let object = m::new_test_struct(12);
        let obj_ref = context::new_object<TestStruct>(ctx, object);
        let test_struct = object::remove(obj_ref);
        m::destroy_test_struct(test_struct);
    }
}

