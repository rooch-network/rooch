// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Context is part of the StorageAbstraction
/// It is used to provide a context for the storage operations, make the storage abstraction, 
/// and let developers customize the storage
module moveos_std::context {

    use moveos_std::storage_context::{StorageContext};
    use moveos_std::tx_context::{TxContext};

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


   
}
