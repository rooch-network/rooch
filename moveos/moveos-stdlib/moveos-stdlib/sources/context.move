// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Context is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers customize the storage
module moveos_std::context {

    use moveos_std::storage_context::{StorageContext};
    use moveos_std::tx_context::{Self, TxContext};

    friend moveos_std::move_module;

    const ErrorObjectOwnerNotMatch: u64 = 1;
    const ErrorObjectNotShared: u64 = 2;
    ///Can not take out the object which is bound to the account
    const ErrorObjectIsBound: u64 = 3;

    /// Information about the global context include TxContext and StorageContext
    /// We can not put the StorageContext to TxContext, because object module depends on tx_context module,
    /// and storage_context module depends on object module.
    /// We put both TxContext and StorageContext to Context, for convenience of developers.
    /// The Context can not be `drop` or `store`, so developers need to pass the `&Context` or `&mut Context` to the `entry` function.
    struct Context {
        tx_context: TxContext,
        /// The Global Object Storage
        storage_context: StorageContext,
    }


    #[test_only]
    /// Create a Context for unit test
    public fun new_test_context(sender: address): Context {
        // We need to ensure the tx_hash is unique, so we append the sender to the seed
        // If a sender create two Context, the tx_hash will be the same.
        // Maybe the test function need to pass a type parameter as seed.
        let seed = b"test_tx";
        std::vector::append(&mut seed, moveos_std::bcs::to_bytes(&sender));
        new_test_context_random(sender, seed)
    }

    #[test_only]
    /// Create a Context for unit test with random seed
    public fun new_test_context_random(sender: address, seed: vector<u8>): Context {
        let tx_context = tx_context::new_test_context_random(sender, seed);
        let storage_context = moveos_std::storage_context::new(&mut tx_context);
        Context {
            tx_context,
            storage_context,
        }
    }

    #[test_only]
    /// Testing only: allow to drop Context
    public fun drop_test_context(self: Context) {
        moveos_std::test_helper::destroy<Context>(self);
    }

   
}
