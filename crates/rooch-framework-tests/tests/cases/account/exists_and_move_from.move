//# init --addresses test=0x42

//# publish
module test::m {
    use std::signer;
    use moveos_std::context::{Context};
    use moveos_std::account;
    use moveos_std::signer as moveos_signer;

    struct Test has key{
        addr: address,
        version: u64
    }

    fun init(ctx: &mut Context) {
        let sender = &moveos_signer::module_signer<Test>();
        let sender_addr = signer::address_of(sender);
        account::move_resource_to(ctx, sender, Test{
            addr: sender_addr,
            version: 0,
        });
    }

    public fun test_exists_and_move_from(ctx: &mut Context, sender:&signer){
        let sender_addr = signer::address_of(sender);
        let test_exists = account::exists_resource<Test>(ctx, sender_addr);
        assert!(test_exists, 1);
        let test = account::move_resource_from<Test>(ctx, sender_addr);
        let test_exists = account::exists_resource<Test>(ctx, sender_addr);
        assert!(!test_exists, 2);
        let Test{
            addr: _,
            version: _
        } = test;
    }
}

//# run --signers test
script {
    use moveos_std::context::{Context};
    use test::m;

    fun main(ctx: &mut Context, sender: signer) {
        m::test_exists_and_move_from(ctx, &sender);
    }
}
