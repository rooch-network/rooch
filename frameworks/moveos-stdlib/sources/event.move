// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::event {
    

    #[private_generics(T)]
    /// Emit a custom Move event, sending the data offchain.
    ///
    /// Used for creating custom indexes and tracking onchain
    /// activity in a way that suits a specific application the most.
    ///
    /// The type T is the main way to index the event, and can contain
    /// phantom parameters, eg. emit(MyEvent<phantom T>).
    public fun emit<T: drop+copy>(event: T) {
        native_emit<T>(event);
    }

    /// Native procedure that writes to the actual event stream in Event store
    native fun native_emit<T>(event: T);

    #[test_only]
    struct WithdrawEvent has drop,copy {
        addr: address,
        amount: u64
    }

    #[test(sender = @0x42)]
    fun test_event(sender: address) {

        emit<WithdrawEvent>(WithdrawEvent {
            addr: sender,
            amount: 100,
        });
        emit<WithdrawEvent>(WithdrawEvent {
            addr: sender,
            amount: 102,
        });

    }

}

