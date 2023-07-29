/// StorageContext is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers can customize the storage

module moveos_std::storage_context {

    use std::option::Option;
    use moveos_std::object_storage::{ObjectStorage};
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object_id::{ObjectID};

    #[test_only]
    use moveos_std::object_storage::{Self};
    #[test_only]
    use moveos_std::test_helper;

    /// Information about the global storage context
    /// We can not put the StorageContext to TxContext, because object module depends on tx_context module,
    /// and storage_context module depends on object module.
    /// We put TxContext to StorageContext, for convenience of developers.
    /// The StorageContext can not be `drop` or `store`, so developers need to pass the `&StorageContext` or `&mut StorageContext` to the `entry` function.
    struct StorageContext {
        tx_context: TxContext,
        /// The Global Object Storage
        object_storage: ObjectStorage,
    }

    /// Get an immutable reference to the transaction context from the storage context
    public fun tx_context(self: &StorageContext): &TxContext {
        &self.tx_context
    }

    /// Get a mutable reference to the transaction context from the storage context
    public fun tx_context_mut(self: &mut StorageContext): &mut TxContext {
        &mut self.tx_context
    }

    /// Get an immutable reference to the object storage from the storage context
    public fun object_storage(self: &StorageContext): &ObjectStorage {
        &self.object_storage
    }

    /// Get a mutable reference to the object storage from the storage context
    public fun object_storage_mut(self: &mut StorageContext): &mut ObjectStorage {
        &mut self.object_storage
    }

    /// Wrap functions for TxContext

    /// Return the address of the user that signed the current transaction
    public fun sender(self: &StorageContext): address {
        tx_context::sender(&self.tx_context)
    } 

    /// Generate a new unique address
    public fun fresh_address(self: &mut StorageContext): address {
        tx_context::fresh_address(&mut self.tx_context)
    }

    /// Generate a new unique object ID
    public fun fresh_object_id(self: &mut StorageContext): ObjectID {
        tx_context::fresh_object_id(&mut self.tx_context)
    }

    /// Return the hash of the current transaction
    public fun tx_hash(self: &StorageContext): vector<u8> {
        tx_context::tx_hash(&self.tx_context)
    } 

    /// Add a value to the context map
    public fun add<T: drop + store + copy>(self: &mut StorageContext, value: T) {
        tx_context::add(&mut self.tx_context, value); 
    }

    /// Get a value from the context map
    public fun get<T: drop + store + copy>(self: &StorageContext): Option<T> {
        tx_context::get(&self.tx_context)
    }

    #[test_only]
    /// Create a StorageContext for unit test
    public fun new_test_context(sender: address): StorageContext {
        let tx_context = tx_context::new_test_context(sender);
        let object_storage = object_storage::new_with_id(object_storage::global_object_storage_handle());
        StorageContext {
            tx_context,
            object_storage,
        }
    }

    #[test_only]
    /// Testing only: allow to drop StorageContext
    public fun drop_test_context(self: StorageContext) {
        test_helper::destroy<StorageContext>(self);
    }
}
