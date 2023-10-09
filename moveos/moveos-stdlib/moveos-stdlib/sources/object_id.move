// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move object identifiers
module moveos_std::object_id {
    friend moveos_std::tx_context;
    friend moveos_std::raw_table;
    friend moveos_std::storage_context;
    friend moveos_std::account_storage;
    friend moveos_std::event;

    /// An object ID
    struct ObjectID has store, copy, drop {
        // TODO should use u256 to replace address?
        id: address,
    }

    /// Generate a new ObjectID from an address
    public(friend) fun address_to_object_id(address: address): ObjectID {
        ObjectID { id: address }
    }
}