// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::core_addresses {
    use std::signer;
    use std::vector;

    /// The operation can only be performed by the VM
    const ErrorNotVM: u64 = 1;
    /// The address is not rooch reserved address
    const ErrorNotSystemReservedAddress: u64 = 2;

    /// Assert that the signer has the VM reserved address.
    public fun assert_vm(account: &signer) {
        assert!(is_vm(account), ErrorNotVM)
    }

    /// Return true if `addr` is a reserved address for the VM to call system modules.
    public fun is_vm(account: &signer): bool {
        is_vm_address(signer::address_of(account))
    }

    /// Return true if `addr` is a reserved address for the VM to call system modules.
    public fun is_vm_address(addr: address): bool {
        addr == @vm_reserved
    }

    public fun assert_system_reserved(account: &signer) {
        assert_system_reserved_address(signer::address_of(account));
    }

    public fun assert_system_reserved_address(addr: address) {
        assert!(
            is_system_reserved_address(addr),
            ErrorNotSystemReservedAddress,
        )
    }

    /// Return true if `addr` is 0x0 or under the on chain governance's control.
    public fun is_system_reserved_address(addr: address): bool {
        // addr == @0x0 ||
        addr == @0x1 ||
        addr == @0x2 ||
        addr == @0x3 ||
        addr == @0x4 ||
        addr == @0x5 ||
        addr == @0x6 ||
        addr == @0x7 ||
        addr == @0x8 ||
        addr == @0x9 ||
        addr == @0xa
    }

    /// Return true if `addr` is either the VM address or an Rooch system address.
    public fun is_reserved_address(addr: address): bool {
        is_system_reserved_address(addr) || is_vm_address(addr)
    }

    /// List all the on chain governance's reserved addresses.
    public fun list_system_reserved_addresses(): vector<address> {
        let addrs = vector::empty<address>();
        vector::push_back(&mut addrs, @0x1);
        vector::push_back(&mut addrs, @0x2);
        vector::push_back(&mut addrs, @0x3);
        vector::push_back(&mut addrs, @0x4);
        vector::push_back(&mut addrs, @0x5);
        vector::push_back(&mut addrs, @0x6);
        vector::push_back(&mut addrs, @0x7);
        vector::push_back(&mut addrs, @0x8);
        vector::push_back(&mut addrs, @0x9);
        vector::push_back(&mut addrs, @0xa);

        addrs
    }
}
