// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// StorageContext is part of the StorageAbstraction
/// TODO we need to redegin the StorageContext and AppStorageContext
module moveos_std::storage_context {

    use moveos_std::object_id::ObjectID;
    friend moveos_std::context;
    
    struct StorageContext has store {
        handle: ObjectID,
    }

    /// Create a new StorageContext with a given ObjectID.
    public(friend) fun new_with_id(handle: ObjectID): StorageContext {
        StorageContext {
            handle,
        }
    }

    #[test_only]
    /// Testing only: allow to drop oject storage
    public fun drop_object_storage(self: StorageContext) {
        moveos_std::test_helper::destroy<StorageContext>(self);
    }

    #[test_only]
    /// There is only one instance: the global object storage.
    /// This `new` function is only used for testing
    public fun new(_ctx: &mut moveos_std::tx_context::TxContext): StorageContext {
        StorageContext {
            handle: moveos_std::object_id::address_to_object_id(@0x0),
        }
    }

    #[test_only]    
    /// Destroy a StorageContext
    public fun destroy_empty(self: StorageContext) {
        let StorageContext { handle:_ } = self;
    }

    #[test_only]
    struct TestObject has key {
        f: u8
    }

    #[test_only]
    struct TestObject2 has key {
        f: u8
    }
 
}
