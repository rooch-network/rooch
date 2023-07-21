//# init --addresses test=0x42

//# publish
module test::m {
    use moveos_std::event;
    use moveos_std::storage_context::StorageContext;
    struct WithdrawEvent{
        addr: address,
        amount: u64
    }

    public fun emit_withdraw_event(ctx: &mut StorageContext, addr: address, amount: u64) {
        let withdraw_event = WithdrawEvent{addr, amount};
        event::emit<WithdrawEvent>(ctx, withdraw_event);
    }
}

//check module exists
//# run --signers test
script {
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;
    use test::m;

    fun main(ctx: &mut StorageContext) {
        let sender_addr = tx_context::sender(storage_context::tx_context(ctx));
        m::emit_withdraw_event(ctx, sender_addr, 100);
    }
}
