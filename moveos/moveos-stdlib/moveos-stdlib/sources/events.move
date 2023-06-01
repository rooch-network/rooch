/// events emit events to the event store.
module moveos_std::events {
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object_id::ObjectID;

    /// Emit a custom Move event, sending the data offchain.
    ///
    /// Used for creating custom indexes and tracking onchain
    /// activity in a way that suits a specific application the most.
    ///
    /// The type T is the main way to index the event, and can contain
    /// phantom parameters, eg emit(MyEvent<phantom T>).
    public fun emit_event<T: drop + store>(ctx: &mut TxContext, event: T) {
        let guid = tx_context::fresh_object_id(ctx);

        //!Notice that: count now just for For placeholder, and value is always 0
        write_to_event_store<T>(&guid, 0,  event);
    }

    /// Native procedure that writes to the actual event stream in Event store
    /// This will replace the "native" portion of EmitEvent bytecode
    native fun write_to_event_store<T: drop + store>(guid: &ObjectID, count: u64, data: T);
}
