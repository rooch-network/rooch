/// `EventHandle`s with unique event handle id (GUID). It contains a counter for the number
/// of `EventHandle`s it generates. An `EventHandle` is used to count the number of
/// events emitted to a handle and emit events to the event store.
module moveos_std::events {
    use std::error;
    use std::bcs;
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context::{Self};
    use moveos_std::object_storage::{Self, ObjectStorage};
    use moveos_std::object_id::{Self, ObjectID};
    use moveos_std::object;
    use moveos_std::type_table::{Self, TypeTable};
    #[test_only]
    use std::debug;
    use std::option::{Self, Option};

    const NamedTableEventHandler: u64 = 2;

    /// The address/account did not correspond to the moveos_std address
    const ENotMoveOSStdAddress: u64 = 0;
    /// The event hander table resource already exists
    const EEventHandlerTableAlreadyExists: u64 = 1;
    /// The event handle with the given type already exists
    const EEventHandleAlreadyExists: u64 = 2;
    /// The event handle with the given type not exists
    const EEventHandleNotExists: u64 = 3;

    struct EventStorage has key {
        event_handle: TypeTable,
    }

    /// A handle for an event such that:
    /// 1. Other modules can emit events to this handle.
    /// 2. Storage can use this handle to prove the total number of events that happened in the past.
    struct EventHandle<phantom T: key> has key, store {
        /// Total number of events emitted to this event stream.
        counter: u64,
        /// A globally unique ID for this event stream. event handler id equal to guid.
        event_handle_id: ObjectID,
        sender: address,
    }

    /// Can only execute by moveos_std account
    fun init(ctx: &mut StorageContext, account: &signer) {
        let account_addr = signer::address_of(account);
        assert!(account_addr == @moveos_std, error::permission_denied(ENotMoveOSStdAddress));
        create_event_storage(ctx, account_addr)
    }

    /// Create a new event storage space
    fun create_event_storage(ctx: &mut StorageContext, account_addr: address) {
        let object_id = derive_event_object_id(account_addr);
        let event_storage = EventStorage {
            event_handle: type_table::new_with_id(object_id),
        };

        let object_storage = storage_context::object_storage_mut(ctx);
        assert!(!object_storage::contains(object_storage, object_id), EEventHandlerTableAlreadyExists);
        let object = object::new_with_id(object_id, account_addr, event_storage);
        object_storage::add(object_storage, object);
    }

    fun derive_event_object_id(account_addr: address): ObjectID{
        object_id::address_to_object_id(tx_context::derive_id(bcs::to_bytes(&account_addr), NamedTableEventHandler))
    }

    //TODO if we create the table every time, how to drop the table?
    fun derive_event_table_handle(object_id: ObjectID): TypeTable{
        // let account_addr = @moveos_std;
        type_table::new_with_id(object_id)
    }

    fun borrow_event_storage(object_storage: &ObjectStorage) : &EventStorage {
        let object_id = derive_event_object_id(@moveos_std);
        let object = object_storage::borrow<EventStorage>(object_storage, object_id);
        object::borrow(object)
    }

    fun borrow_mut_event_storage(object_storage: &mut ObjectStorage) : &mut EventStorage {
        let object_id = derive_event_object_id(@moveos_std);
        let object = object_storage::borrow_mut<EventStorage>(object_storage, object_id);
        object::borrow_mut(object)
    }

    /// Add a event handle to the event storage
    fun add_event_handle_to_event_storage<T: key>(object_storage: &mut ObjectStorage, event_handle_resource: T){
        // let event_table_handle = derive_event_table_handle();
        let event_storage = borrow_mut_event_storage(object_storage);
        assert!(!type_table::contains_internal<T>(&event_storage.event_handle), EEventHandleAlreadyExists);
        type_table::add_internal(&mut event_storage.event_handle, event_handle_resource);
    }

    fun exists_event_handle_at_event_storage<T: key>(object_storage: &ObjectStorage) : bool {
        // let event_table_handle = derive_event_table_handle();
        let event_storage = borrow_event_storage(object_storage);
        type_table::contains<T>(&event_storage.event_handle)
    }

    /// Borrow a mut event handle from the event storage
    fun borrow_event_handle_from_event_storage<T: key>(object_storage: &ObjectStorage): &T {
        let event_storage = borrow_event_storage(object_storage);
        type_table::borrow_internal<T>(&event_storage.event_handle)
    }

    /// Borrow a mut event handle from the event storage
    fun borrow_mut_event_handle_from_event_storage<T: key>(object_storage: &mut ObjectStorage): &mut T {
        let event_storage = borrow_mut_event_storage(object_storage);
        type_table::borrow_mut_internal<T>(&mut event_storage.event_handle)
    }

    /// use query this method to get event handle Metadata
    /// is event_handle_id doesn't exist, sender will default 0x0
    public fun get_event_handle<T: key>(ctx: &StorageContext): (Option<ObjectID>, address, u64){
        if (exists_event_handle_at_event_storage<EventHandle<T>>(storage_context::object_storage(ctx))) {
            let event_handle = borrow_event_handle_from_event_storage<EventHandle<T>>(storage_context::object_storage(ctx));
            (option::some(event_handle.event_handle_id), event_handle.sender, event_handle.counter)
        } else {
            (option::none<ObjectID>(), @0x0, 0)
        }
    }

    /// Use EventHandle to generate a unique event handle
    /// user doesn't need to call this method directly
    fun new_event_handle<T: key>(ctx: &mut StorageContext) {
        let account_addr = tx_context::sender(storage_context::tx_context(ctx));
        let event_handle_id = tx_context::fresh_object_id(storage_context::tx_context_mut(ctx));
        let event_handle = EventHandle<T> {
            counter: 0,
            event_handle_id,
            sender: account_addr,
        };
        add_event_handle_to_event_storage<EventHandle<T>>(storage_context::object_storage_mut(ctx), event_handle)
    }

    public fun ensure_event_handle<T: key>(ctx: &mut StorageContext) {
        if (!exists_event_handle_at_event_storage<EventHandle<T>>(storage_context::object_storage(ctx))) {
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
        let event_handle_ref = borrow_mut_event_handle_from_event_storage<EventHandle<T>>(storage_context::object_storage_mut(ctx));
        emit<T>(&event_handle_ref.event_handle_id, event_handle_ref.counter, event);
        event_handle_ref.counter = event_handle_ref.counter + 1;
    }

    /// Native procedure that writes to the actual event stream in Event store
    native fun emit<T: key>(event_handle_id: &ObjectID, count: u64, event: T);


    /// Destroy a unique handle.
    public fun destroy_handle<T: key>(handle: EventHandle<T>) {
        EventHandle<T> { counter: _, event_handle_id: _, sender: _} = handle;
    }

    #[test]
    fun test_named_table_id() {
        debug::print(&derive_event_object_id(@moveos_std));
        assert!(derive_event_object_id(@moveos_std) == object_id::address_to_object_id(@0xd7f6abc69aa5cc17b99aeb86fd14d4361e7d3ffd1baacab90eb3765785d3515d), 1000);
    }

    #[test_only]
    struct WithdrawEvent has key{
        addr: address,
        amount: u64
    }

    #[test(sender=@0x1)]
    fun test_event_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);

        //init event storage, only for test
        init(&mut ctx, &sender);
        // create_event_storage(&mut ctx, sender_addr);

        new_event_handle<WithdrawEvent>(&mut ctx);
        let (event_hanlde_id, event_sender_addr, event_seq) = get_event_handle<WithdrawEvent>(&ctx);
        debug::print(&event_hanlde_id);
        debug::print(&event_sender_addr);
        debug::print(&event_seq);

        emit_event<WithdrawEvent>(&mut ctx, WithdrawEvent {
            addr: signer::address_of(&sender),
            amount: 100,
        });
        emit_event<WithdrawEvent>(&mut ctx, WithdrawEvent {
            addr: signer::address_of(&sender),
            amount: 102,
        });

        storage_context::drop_test_context(ctx);
    }
}
