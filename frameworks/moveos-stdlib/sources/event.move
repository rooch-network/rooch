// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::event {
    use moveos_std::object;
    use moveos_std::object::{ObjectID};

    public fun named_event_handle_id<T>(): ObjectID {
        object::named_object_id<T>()
    }

    public fun custom_event_handle_id<ID: store + copy + drop, T: key>(id: ID): ObjectID {
        object::custom_object_id<ID, T>(id)
    }

    #[private_generics(T)]
    /// Emit a custom Move event, sending the data offchain.
    ///
    /// Used for creating custom indexes and tracking onchain
    /// activity in a way that suits a specific application the most.
    ///
    /// The type T is the main way to index the event, and can contain
    /// phantom parameters, eg. emit(MyEvent<phantom T>).
    public fun emit<T: drop + copy>(event: T) {
        let event_handle_id = named_event_handle_id::<T>();
        native_emit_with_handle<T>(event, event_handle_id);
    }

    #[private_generics(T)]
    /// Emit a custom Move event with handle
    public fun emit_with_handle<T: drop + copy>(event: T, event_handle_id: ObjectID) {
        native_emit_with_handle<T>(event, event_handle_id);
    }

    /// Native procedure that writes to the actual event stream in Event store
    native fun native_emit<T>(event: T);

    /// Native procedure that writes to the actual event stream in Event store with handle
    native fun native_emit_with_handle<T>(event: T, event_handle_id: ObjectID);

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

