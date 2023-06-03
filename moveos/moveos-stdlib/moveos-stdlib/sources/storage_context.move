/// StorageContext is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers can customize the storage

module moveos_std::storage_context {
    use moveos_std::object_storage::{ObjectStorage};
    use moveos_std::tx_context::{TxContext};

    #[test_only]
    use moveos_std::object_storage::{Self};
    #[test_only]
    use moveos_std::tx_context::{Self};
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

    public fun tx_context(this: &StorageContext): &TxContext {
        &this.tx_context
    }

    public fun tx_context_mut(this: &mut StorageContext): &mut TxContext {
        &mut this.tx_context
    }

    public fun object_storage(this: &StorageContext): &ObjectStorage {
        &this.object_storage
    }

    public fun object_storage_mut(this: &mut StorageContext): &mut ObjectStorage {
        &mut this.object_storage
    }

    //TODO should we wrapper the tx_context::fresh_object_id at here?

    #[test_only]
    /// Create a StorageContext and AccountStorage for unit test
    public fun new_test_context(sender: address): StorageContext {
        let tx_context = tx_context::new_test_context(sender);
        // let object_storage = object_storage::new(&mut tx_context);
        let object_storage = object_storage::new_with_id(object_storage::global_object_storage_handle());
        StorageContext {
            tx_context,
            object_storage,
        }
    }

    #[test_only]
    /// Testing only: allow to drop oject storage
    public fun drop_test_context(this: StorageContext) {
        test_helper::destroy<StorageContext>(this);
    }
}