/// StorageContext is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers can customize the storage

module moveos_std::storage_context {
    use moveos_std::object::{ObjectStore};
    use moveos_std::tx_context::{TxContext};

    /// Information about the global storage context
    /// We can not put the StorageContext to TxContext, because object module depends on tx_context module,
    /// and storage_context module depends on object module.
    /// We put TxContext to StorageContext, for convenience of developers.
    /// The StorageContext can not be `drop` or `store`, so developers need to pass the `&StorageContext` or `&mut StorageContext` to the `entry` function.
    struct StorageContext {
        tx_context: TxContext,
        /// The Global Object Storage
        object_store: ObjectStore,
    }

    public fun tx_context(this: &StorageContext): &TxContext {
        &this.tx_context
    }

    public fun tx_context_mut(this: &mut StorageContext): &mut TxContext {
        &mut this.tx_context
    }

    public fun object_store(this: &StorageContext): &ObjectStore {
        &this.object_store
    }

    public fun object_store_mut(this: &mut StorageContext): &mut ObjectStore {
        &mut this.object_store
    }

}