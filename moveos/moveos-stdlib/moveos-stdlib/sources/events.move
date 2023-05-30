/// The Event module defines an `EventHandleGenerator` that is used to create
/// `EventHandle`s with unique GUIDs. It contains a counter for the number
/// of `EventHandle`s it generates. An `EventHandle` is used to count the number of
/// events emitted to a handle and emit events to the event store.
module moveos_std::events {
    use std::error;
    use std::bcs;
    use std::signer;
    use std::vector;
    use moveos_std::account_storage;
    use moveos_std::storage_context::{StorageContext};

    /// A resource representing the counter used to generate uniqueness under each account. There won't be destructor for
    /// this resource to guarantee the uniqueness of the generated handle.
    struct EventHandleGenerator has key {
        // A monotonically increasing counter
        counter: u64,
        addr: address,
    }

    /// A handle for an event such that:
    /// 1. Other modules can emit events to this handle.
    /// 2. Storage can use this handle to prove the total number of events that happened in the past.
    struct EventHandle<phantom T: drop + store> has store {
        /// Total number of events emitted to this event stream.
        counter: u64,
        /// A globally unique ID for this event stream.
        guid: vector<u8>,
    }

    /// The event generator resource was in an invalid state
    const EEventGenerator: u64 = 0;

    /// Publishs a new event handle generator.
    public fun publish_generator(ctx: &mut StorageContext, account: &signer) {
        let addr = signer::address_of(account);
        assert!(!account_storage::global_exists<EventHandleGenerator>(ctx, addr), error::already_exists(EEventGenerator));
        account_storage::global_move_to(
            ctx,
            account,
            EventHandleGenerator{ counter: 0, addr }
        )
    }

    /// Derive a fresh unique id by using sender's EventHandleGenerator. The generated vector<u8> is indeed unique because it
    /// was derived from the hash(sender's EventHandleGenerator || sender_address). This module guarantees that the
    /// EventHandleGenerator is only going to be monotonically increased and there's no way to revert it or destroy it. Thus
    /// such counter is going to give distinct value for each of the new event stream under each sender. And since we
    /// hash it with the sender's address, the result is guaranteed to be globally unique.
    fun fresh_guid(counter: &mut EventHandleGenerator): vector<u8> {
        let sender_bytes = bcs::to_bytes(&counter.addr);
        let count_bytes = bcs::to_bytes(&counter.counter);
        counter.counter = counter.counter + 1;

        // EventHandleGenerator goes first just in case we want to extend address in the future.
        vector::append(&mut count_bytes, sender_bytes);

        count_bytes
    }

    /// Use EventHandleGenerator to generate a unique event handle
    public fun new_event_handle<T: drop + store>(ctx: &mut StorageContext, account: &signer): EventHandle<T> {
        let addr = signer::address_of(account);
        assert!(account_storage::global_exists<EventHandleGenerator>(ctx, addr), error::not_found(EEventGenerator));
        EventHandle<T> {
            counter: 0,
            guid: fresh_guid(account_storage::global_borrow_mut<EventHandleGenerator>(ctx, addr))
        }
    }

    /// Emit an event with payload `data` by using `handle_ref`'s key and counter.
    public fun emit_event<T: drop + store>(handle_ref: &mut EventHandle<T>, data: T) {
        let guid = *&handle_ref.guid;

        write_to_event_store<T>(guid, handle_ref.counter, data);
        handle_ref.counter = handle_ref.counter + 1;
    }

    /// Native procedure that writes to the actual event stream in Event store
    /// This will replace the "native" portion of EmitEvent bytecode
    native fun write_to_event_store<T: drop + store>(guid: vector<u8>, count: u64, data: T);

    /// Destroy a unique handle.
    public fun destroy_handle<T: drop + store>(handle: EventHandle<T>) {
        EventHandle<T> { counter: _, guid: _ } = handle;
    }
}
