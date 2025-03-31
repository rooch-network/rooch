// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::event {
    use moveos_std::object;
    use moveos_std::object::{ObjectID};

    public fun named_event_handle_id<T>(): ObjectID {
        object::named_object_id<T>()
    }

    public fun custom_event_handle_id<ID: store + copy + drop, T>(id: ID): ObjectID {
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
        let event_handle_id = named_event_handle_id<T>();
        native_emit_with_handle<T>(event_handle_id, event);
    }

    #[private_generics(T)]
    /// Emit a custom Move event with handle
    public fun emit_with_handle<T: drop + copy>(event_handle_id: ObjectID, event: T) {
        native_emit_with_handle<T>(event_handle_id, event);
    }

    /// Native procedure that writes to the actual event stream in Event store
    native fun native_emit<T>(event: T);

    /// Native procedure that writes to the actual event stream in Event store with handle
    native fun native_emit_with_handle<T>(event_handle_id: ObjectID, event: T);

    #[test_only]
    struct WithdrawEvent has drop,copy {
        addr: address,
        amount: u64
    }

    #[test_only]
    struct DepositEvent has drop, copy, key {
        addr: address,
        amount: u64
    }

    #[test(sender = @0x42)]
    fun test_emit(sender: address) {
        // Test basic emit functionality
        emit<WithdrawEvent>(WithdrawEvent {
            addr: sender,
            amount: 100,
        });
        emit<WithdrawEvent>(WithdrawEvent {
            addr: sender,
            amount: 102,
        });
    }

    #[test(sender = @0x42)]
    fun test_emit_with_handle(sender: address) {
        // Test emit_with_handle functionality
        let custom_id = x"1234";
        let event_handle_id = custom_event_handle_id<vector<u8>, DepositEvent>(custom_id);
        
        emit_with_handle<DepositEvent>(
            event_handle_id,
            DepositEvent {
                addr: sender,
                amount: 200,
            },
        );

        // Test with same handle but different event
        emit_with_handle<DepositEvent>(
            event_handle_id,
            DepositEvent {
                addr: sender,
                amount: 300,
            },
        );

        // Test with named event handle
        let named_handle_id = named_event_handle_id<DepositEvent>();
        emit_with_handle<DepositEvent>(
            named_handle_id,
            DepositEvent {
                addr: sender,
                amount: 400,
            },
        );
    }

}

