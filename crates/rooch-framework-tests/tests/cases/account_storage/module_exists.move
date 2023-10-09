//# init --addresses test=0x42

//# publish
module test::m {
}

//check module exists
//# run --signers test
script {
    use std::string::{Self};
    use moveos_std::account_storage;
    use moveos_std::context::{Context};

    fun main(ctx: &mut Context) {
        assert!(account_storage::exists_module(ctx, @moveos_std, string::utf8(b"account_storage")), 0);
        assert!(account_storage::exists_module(ctx, @test, string::utf8(b"m")), 1);
    }
}
