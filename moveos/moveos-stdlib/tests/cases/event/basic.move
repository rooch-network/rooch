//# init --addresses test=0x42

//# publish
module test::m {
    struct WithdrawEvent has key, copy{
        // struct WithdrawEvent has key{
        addr: address,
        amount: u64
    }
}

//check module exists
//# run --signers test
script {
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;
    use moveos_std::events;
    use test::m::WithdrawEvent;

    fun main(ctx: &mut StorageContext) {
        // assert!(account_storage::exists_module(ctx, @0x1, string::utf8(b"account_storage")), 0);
        // assert!(account_storage::exists_module(ctx, @test, string::utf8(b"m")), 1);

        let _sender_addr = tx_context::sender(storage_context::tx_context(ctx));
        // emit_event<WithdrawEvent>(&mut ctx, WithdrawEvent {
        //     addr: signer::address_of(&sender),
        //     amount: 100,
        // });
        events::emit_event<WithdrawEvent>(ctx, WithdrawEvent {
            addr: sender_addr,
            amount: 102,
        });

        // let (event_hanlde_id, event_sender_addr, event_seq) = get_event_handle<WithdrawEvent>(&ctx);
        // debug::print(&event_hanlde_id);
        // debug::print(&event_sender_addr);
        // debug::print(&event_seq);
    }
}
