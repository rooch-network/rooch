//# init --addresses test=0x42

//# publish
module test::m {
    use moveos_std::event;
    struct WithdrawEvent has drop,copy {
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
    
    use test::m;

    fun main() {
        let sender_addr = moveos_std::tx_context::sender();
        m::emit_withdraw_event(sender_addr, 100);
    }
}
