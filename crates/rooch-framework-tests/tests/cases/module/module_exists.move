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
        assert!(module_store::exists_package(@moveos_std), 0);
        assert!(module_store::exists_module(@moveos_std, string::utf8(b"module_store")), 1);
        assert!(module_store::exists_package(@test), 2);
        assert!(module_store::exists_module(@test, string::utf8(b"m")), 3);
    }
}