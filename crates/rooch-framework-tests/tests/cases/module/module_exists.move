//# init --addresses test=0x42

//# publish
module test::m {
}

//check module exists
//# run --signers test
script {
    use std::string::{Self};
    use moveos_std::context::{Self, Context};

    fun main(ctx: &mut Context) {
        assert!(context::exists_module(ctx, @moveos_std, string::utf8(b"move_module")), 0);
        assert!(context::exists_module(ctx, @test, string::utf8(b"m")), 1);
    }
}
