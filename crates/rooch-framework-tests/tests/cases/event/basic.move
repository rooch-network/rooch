//# init --addresses test=0x42

//# publish
module test::m {
    use moveos_std::event;
    use moveos_std::context::Context;
    struct WithdrawEvent{
        addr: address,
        amount: u64
    }

    public fun emit_withdraw_event(addr: address, amount: u64) {
        let withdraw_event = WithdrawEvent{addr, amount};
        event::emit<WithdrawEvent>(withdraw_event);
    }
}

//check module exists
//# run --signers test
script {
    use moveos_std::context::{Self, Context};
    use test::m;

    fun main(ctx: &mut Context) {
        let sender_addr = context::sender(ctx);
        m::emit_withdraw_event(sender_addr, 100);
    }
}
