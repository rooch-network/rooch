module rooch_examples::event_test {
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::event;
    use moveos_std::tx_context;

    #[test_only]
    use std::debug;


    struct WithdrawEvent {
        addr: address,
        amount: u64
    }

    public entry fun emit_event(
        ctx: &mut StorageContext,
        // addr: address,
        amount: u64,
    ) {
        let addr = tx_context::sender(storage_context::tx_context(ctx));
        event::emit<WithdrawEvent>(ctx, WithdrawEvent {
            addr,
            amount,
        });
    }

    #[test]
    fun test_get_test_event_handle() {
        let event_handle_id = event::derive_event_handle_id<WithdrawEvent>();
        debug::print(&120120);
        debug::print(&event_handle_id);
    }

    // #[test(sender = @042)]
    // fun test_event_emit(sender: signer) {
    //     let sender_addr = signer::address_of(&sender);
    //     let ctx = storage_context::new_test_context(sender_addr);
    //
    //     event::emit<WithdrawEvent>(&mut ctx, WithdrawEvent {
    //         addr: signer::address_of(&sender),
    //         amount: 100,
    //     });
    //
    //     storage_context::drop_test_context(ctx);
    // }
}
