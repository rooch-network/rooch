// Origin source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/tx_context.move#L24
// And do refactoring

module moveos_std::tx_context {
    use std::vector;
    use std::hash;
    use std::string::String;
    use std::option::{Self, Option};
    use std::error;
    use moveos_std::bcs;
    use moveos_std::object_id::{Self, ObjectID};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::copyable_any::{Self, Any};
    use moveos_std::type_info;
    use moveos_std::tx_meta::{TxMeta};

    friend moveos_std::object;
    friend moveos_std::raw_table;
    friend moveos_std::account_storage;
    friend moveos_std::event;

    const EInvalidContext: u64 = 1;

    /// Information about the transaction currently being executed.
    /// This cannot be constructed by a transaction--it is a privileged object created by
    /// the VM and passed in to the entrypoint of the transaction as `&mut TxContext`.
    struct TxContext has drop {
        /// The address of the user that signed the current transaction
        sender: address,
        /// Hash of the current transaction
        tx_hash: vector<u8>,
        /// Counter recording the number of fresh id's created while executing
        /// this transaction. Always 0 at the start of a transaction
        ids_created: u64,
        /// A Key-Value map that can be used to store context information
        map: SimpleMap<String, Any>,
    }

    /// Return the address of the user that signed the current
    /// transaction
    public fun sender(self: &TxContext): address {
        self.sender
    } 

    /// Generate a new unique address,
    public fun fresh_address(ctx: &mut TxContext): address {
        let addr = derive_id(ctx.tx_hash, ctx.ids_created);
        ctx.ids_created = ctx.ids_created + 1;
        addr
    }

    /// Generate a new unique object ID
    public fun fresh_object_id(ctx: &mut TxContext): ObjectID {
        object_id::address_to_object_id(fresh_address(ctx))
    }

    public(friend) fun derive_id(hash: vector<u8>, index: u64): address {
        let bytes = hash;
        vector::append(&mut bytes, bcs::to_bytes(&index));
        //TODO change return type to h256 and use h256 to replace address?
        let id = hash::sha3_256(bytes);
        bcs::to_address(id)
    }

    /// Return the hash of the current transaction
    public fun tx_hash(self: &TxContext): vector<u8> {
        self.tx_hash
    }

    /// Return the number of id's created by the current transaction.
    /// Hidden for now, but may expose later
    fun ids_created(self: &TxContext): u64 {
        self.ids_created
    }

    /// Add a value to the context map
    public fun add<T: drop + store + copy>(self: &mut TxContext, value: T) {
        let any = copyable_any::pack(value);
        let type_name = *copyable_any::type_name(&any);
        simple_map::add(&mut self.map, type_name, any)
    }

    /// Get a value from the context map
    public fun get<T: drop + store + copy>(self: &TxContext): Option<T> {
        let type_name = type_info::type_name<T>();
        if (simple_map::contains_key(&self.map, &type_name)) {
            let any = simple_map::borrow(&self.map, &type_name);
            option::some(copyable_any::unpack(*any))   
        }else{
            option::none()
        }
    }

    /// Check if the key is in the context map
    public fun contains<T: drop + store + copy>(self: &TxContext): bool {
        let type_name = type_info::type_name<T>();
        simple_map::contains_key(&self.map, &type_name)
    }

    /// Get the transaction meta data
    /// The TxMeta is writed by the VM before the transaction execution.
    /// The meta data is only available when executing or validating a transaction, otherwise abort(eg. readonly function call).
    public fun tx_meta(self: &TxContext): TxMeta {
        let meta = get<TxMeta>(self);
        assert!(option::is_some(&meta), error::invalid_state(EInvalidContext));
        option::extract(&mut meta)
    }

    #[test_only]
    /// Create a TxContext for unit test
    public fun new_test_context(sender: address): TxContext {
        let tx_hash = hash::sha3_256(b"test_tx");
        TxContext {
            sender,
            tx_hash,
            ids_created: 0,
            map: simple_map::create(),
        }
    }

    #[test_only]
    struct TestValue has store, drop, copy{
        value: u64,
    }

    #[test(sender=@0x42)]
    fun test_context(sender: address) {
        let ctx = new_test_context(sender);
        let value = TestValue{value: 42};
        add(&mut ctx, value);
        let value2 = get<TestValue>(&ctx);
        assert!(value == option::extract(&mut value2), 1000);
    }
}
