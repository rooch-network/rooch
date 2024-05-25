//# init --addresses test=0x42

//# publish
module test::m {
}

//check module exists
//# run --signers test
script {
    use std::string::{Self};
    use moveos_std::module_store;

    fun main() {
        let module_store = borrow_module_store();
        assert!(module_store::exists_module(module_store, @moveos_std, string::utf8(b"module_store")), 0);
        assert!(module_store::exists_module(module_store, @test, string::utf8(b"m")), 1);
    }
}
