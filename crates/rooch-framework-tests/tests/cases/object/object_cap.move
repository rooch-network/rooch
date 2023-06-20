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
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;
    use moveos_std::object;
    use test::m::{Self, TestObject};

    fun main(ctx: &mut StorageContext) {
        let sender_addr = tx_context::sender(storage_context::tx_context(ctx));
        let object = m::new_test_object(12);
        let obj = object::new<TestObject>(storage_context::tx_context_mut(ctx), sender_addr, object);

        let _borrow_object = object::borrow(&obj);
        let (_id, _owner, test_object) = object::unpack(obj);
        m::destroy_test_object(test_object);
    }
}
// check: ABORTED

