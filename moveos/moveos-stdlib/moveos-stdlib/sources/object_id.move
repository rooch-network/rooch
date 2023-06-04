/// Move object identifiers
module moveos_std::object_id {
    use std::hash;
    use moveos_std::bcd;

    friend moveos_std::tx_context;
    friend moveos_std::raw_table;
    friend moveos_std::object_storage;
    friend moveos_std::account_storage;
    friend moveos_std::events;

    struct ObjectID has store, copy, drop {
        //TODO should use u256 to replace address?
        id: address,
    }

    public(friend) fun address_to_object_id(address: address): ObjectID {
        ObjectID{id: address}
    }

    public(friend) fun bytes_to_object_id(v: vector<u8>): ObjectID {
        let id = hash::sha3_256(v);
        let addr = bcd::to_address(id);
        ObjectID{id: addr}
    }
}