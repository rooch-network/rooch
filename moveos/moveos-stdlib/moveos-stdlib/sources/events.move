/// `EventHandle`s with unique event handle id (GUID). It contains a counter for the number
/// of `EventHandle`s it generates. An `EventHandle` is used to count the number of
/// events emitted to a handle and emit events to the event store.
module moveos_std::events {
    use std::bcs;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context::{Self};
    use moveos_std::object_storage::{Self, ObjectStorage};
    use moveos_std::object_id::{Self, ObjectID};
    use moveos_std::object;
    #[test_only]
    use std::debug;
    #[test_only]
    use std::signer;
    use std::hash;
    use moveos_std::type_info;
    use moveos_std::bcd;

    /// A handle for an event such that:
    /// 1. Other modules can emit events to this handle.
    /// 2. Storage can use this handle to prove the total number of events that happened in the past.
    struct EventHandle has key, store {
        /// Total number of events emitted to this event stream.
        counter: u64,
    }

    /// A globally unique ID for this event stream. event handler id equal to guid.
    public fun derive_event_handle_id<T: key>(): ObjectID {
        let type_info = type_info::type_of<T>();
        let event_handle_address = bcd::to_address(hash::sha3_256(bcs::to_bytes(&type_info)));
        object_id::address_to_object_id(event_handle_address)
    }

    fun exists_event_handle<T: key>(object_storage: &ObjectStorage): bool {
        let event_handle_id = derive_event_handle_id<T>();
        object_storage::contains(object_storage, event_handle_id)
    }

    /// Borrow a mut event handle from the object storage
    fun borrow_event_handle<T: key>(object_storage: &ObjectStorage): &EventHandle {
        let event_handle_id = derive_event_handle_id<T>();
        let object = object_storage::borrow<EventHandle>(object_storage, event_handle_id);
        object::borrow(object)
    }

    /// Borrow a mut event handle from the object storage
    fun borrow_event_handle_mut<T: key>(object_storage: &mut ObjectStorage): &mut EventHandle {
        let event_handle_id = derive_event_handle_id<T>();
        let object = object_storage::borrow_mut<EventHandle>(object_storage, event_handle_id);
        object::borrow_mut(object)
    }

    /// Get event handle owner
    fun get_event_handle_owner<T: key>(object_storage: &ObjectStorage): address {
        let event_handle_id = derive_event_handle_id<T>();
        let object = object_storage::borrow<EventHandle>(object_storage, event_handle_id);
        object::owner(object)
    }

    /// use query this method to get event handle Metadata
    /// is event_handle_id doesn't exist, sender will default 0x0
    public fun get_event_handle<T: key>(ctx: &StorageContext): (ObjectID, address, u64) {
        let event_handle_id = derive_event_handle_id<T>();
        let sender = @0x0;
        let event_seq = 0;
        if (exists_event_handle<T>(storage_context::object_storage(ctx))) {
            let event_handle = borrow_event_handle<T>(
                storage_context::object_storage(ctx)
            );
            event_seq = event_handle.counter;
            sender = get_event_handle_owner<T>(storage_context::object_storage(ctx));
        };
        (event_handle_id, sender, event_seq)
    }

    /// Use EventHandle to generate a unique event handle
    /// user doesn't need to call this method directly
    fun new_event_handle<T: key>(ctx: &mut StorageContext) {
        let account_addr = tx_context::sender(storage_context::tx_context(ctx));
        let event_handle_id = derive_event_handle_id<T>();
        let event_handle = EventHandle {
            counter: 0,
        };
        let object = object::new_with_id<EventHandle>(event_handle_id, account_addr, event_handle);
        object_storage::add(storage_context::object_storage_mut(ctx), object)
    }

    public fun ensure_event_handle<T: key>(ctx: &mut StorageContext) {
        if (!exists_event_handle<T>(storage_context::object_storage(ctx))) {
            new_event_handle<T>(ctx);
        }
    }

    /// Emit a custom Move event, sending the data offchain.
    ///
    /// Used for creating custom indexes and tracking onchain
    /// activity in a way that suits a specific application the most.
    ///
    /// The type T is the main way to index the event, and can contain
    /// phantom parameters, eg emit(MyEvent<phantom T>).
    public fun emit_event<T: key>(ctx: &mut StorageContext, event: T) {
        ensure_event_handle<T>(ctx);
        let event_handle_id = derive_event_handle_id<T>();
        let event_handle_ref = borrow_event_handle_mut<T>(
            storage_context::object_storage_mut(ctx)
        );
        emit<T>(&event_handle_id, event_handle_ref.counter, event);
        event_handle_ref.counter = event_handle_ref.counter + 1;
    }

    /// Native procedure that writes to the actual event stream in Event store
    native fun emit<T: key>(event_handle_id: &ObjectID, count: u64, event: T);

    #[test_only]
    struct WithdrawEvent has key {
        addr: address,
        amount: u64
    }

    #[test(sender = @0x1)]
    fun test_event(sender: signer) {
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);

        emit_event<WithdrawEvent>(&mut ctx, WithdrawEvent {
            addr: signer::address_of(&sender),
            amount: 100,
        });
        emit_event<WithdrawEvent>(&mut ctx, WithdrawEvent {
            addr: signer::address_of(&sender),
            amount: 102,
        });

        let (event_hanlde_id, event_sender_addr, event_seq) = get_event_handle<WithdrawEvent>(&ctx);
        debug::print(&event_hanlde_id);
        debug::print(&event_sender_addr);
        debug::print(&event_seq);

        storage_context::drop_test_context(ctx);
    }

    #[test]
    fun test_bytes_to_object_id() {
        let event_handle_id = derive_event_handle_id<WithdrawEvent>();
        debug::print(&200200);
        debug::print(&event_handle_id);
    }
}

