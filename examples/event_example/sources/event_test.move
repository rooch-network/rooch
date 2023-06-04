module rooch_examples::event_test {
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::events;
    use moveos_std::tx_context;

    #[test_only]
    use std::hash;
    #[test_only]
    use std::bcs;
    #[test_only]
    use moveos_std::type_info;
    #[test_only]
    use std::bcd;
    #[test_only]
    use std::debug;
    // #[test_only]
    // use std::signer;


    struct WithdrawEvent has key {
        addr: address,
        amount: u64
    }

    public entry fun emit_event(
        ctx: &mut StorageContext,
        // addr: address,
        amount: u64,
    ) {
        let addr = tx_context::sender(storage_context::tx_context(ctx));
        events::emit_event<WithdrawEvent>(ctx, WithdrawEvent {
            addr,
            amount,
        });
    }

    #[test]
    fun test_get_test_event_handle() {
        let bytes = hash::sha3_256(bcs::to_bytes(&type_info::type_of<WithdrawEvent>()));
        let id = hash::sha3_256(bytes);
        let addr = bcd::to_address(id);
        debug::print(&addr);
    }

    // #[test(sender = @042)]
    // fun test_event_emit(sender: signer) {
    //     let sender_addr = signer::address_of(&sender);
    //     let ctx = storage_context::new_test_context(sender_addr);
    //
    //     events::emit_event<WithdrawEvent>(&mut ctx, WithdrawEvent {
    //         addr: signer::address_of(&sender),
    //         amount: 100,
    //     });
    //
    //     storage_context::drop_test_context(ctx);
    // }
}
