//# init --addresses test=0x42

//# publish
module test::m {
}

//check module exists
//# run --signers test
script {
    use std::string::{Self};
    use moveos_std::account_storage;
    use moveos_std::storage_context::{StorageContext};

    fun main(ctx: &mut StorageContext) {
        assert!(account_storage::exists_module(ctx, @0x1, string::utf8(b"account_storage")), 0);
        assert!(account_storage::exists_module(ctx, @test, string::utf8(b"m")), 1);
    }
}
