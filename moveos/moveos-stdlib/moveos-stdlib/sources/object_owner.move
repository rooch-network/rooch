module moveos_std::object_owner {
    use std::option::Option;
    use std::option;

    friend moveos_std::object;

    const ADDRESS_OWNER: u8 = 0;
    const IMMUTABLE: u8 = 1;
    const SHARED: u8 = 2;
    //TODO define object owner?

    struct Owner has copy, drop {
        type: u8,
        address: Option<address>,
    }

    public(friend) fun address_owner(address: address): Owner {
        Owner { type: ADDRESS_OWNER, address: option::some(address) }
    }

    public(friend) fun immutable(): Owner {
        Owner { type: IMMUTABLE, address: option::none() }
    }

    public(friend) fun shared(): Owner {
        Owner { type: SHARED, address: option::none() }
    }

    public fun is_address_owner(owner: Owner): bool {
        owner.type == ADDRESS_OWNER
    }

    public fun is_immutable(owner: Owner): bool {
        owner.type == IMMUTABLE
    }

    public fun is_shared(owner: Owner): bool {
        owner.type == SHARED
    }

    public fun owner_address(owner: Owner): address {
        option::destroy_some(owner.address)
    }
}
