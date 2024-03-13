// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::event_test {
    
    use moveos_std::event;

    struct WithdrawEvent has copy, drop {
        addr: address,
        amount: u64
    }

    public entry fun emit_event(
        
        amount: u64,
    ) {
        let addr = moveos_std::tx_context::sender();
        event::emit<WithdrawEvent>(WithdrawEvent {
            addr,
            amount,
        });
    }

    #[test(sender = @042)]
    fun test_event_emit(sender: address) {
    
        event::emit<WithdrawEvent>(WithdrawEvent {
            addr: sender,
            amount: 100,
        });
    }
}
