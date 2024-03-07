//# init --addresses test=0x42

//# publish
module test::m {
}

//check module exists
//# run --signers test
script {
    use std::string::{Self};
    use moveos_std::move_module;

    fun main() {
        let module_store = borrow_module_store();
        assert!(move_module::exists_module(module_store, @moveos_std, string::utf8(b"move_module")), 0);
        assert!(move_module::exists_module(module_store, @test, string::utf8(b"m")), 1);
    }
}
