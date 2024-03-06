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
        assert!(move_module::exists_module(@moveos_std, string::utf8(b"move_module")), 0);
        assert!(move_module::exists_module(@test, string::utf8(b"m")), 1);
    }
}
