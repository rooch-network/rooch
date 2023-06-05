//# init --addresses test=0x42

//# publish
module test::m {
    struct WithdrawEvent has key{
        addr: address,
        amount: u64
    }

    public fun new_test_struct(addr: address, amount: u64): WithdrawEvent {
        WithdrawEvent{
            addr,
            amount,
        }
    }
}

//check module exists
//# run --signers test
script {
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;
    use moveos_std::events;
    use test::m::{Self, WithdrawEvent};

    fun main(ctx: &mut StorageContext) {
        let sender_addr = tx_context::sender(storage_context::tx_context(ctx));
        let withdraw_event = m::new_test_struct(sender_addr, 100);
        events::emit_event<WithdrawEvent>(ctx, withdraw_event);
    }
}
