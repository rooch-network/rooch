// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::core_addresses {
    use std::signer;

    /// The address/account did not correspond to the genesis address
    const ErrorNotGenesisAddress: u64 = 1;
    /// The address/account did not correspond to the core framework address
    const ErrorNotRoochFrameworkAddress: u64 = 2;

    public fun assert_rooch_genesis(account: &signer) {
        assert_rooch_genesis_address(signer::address_of(account))
    }

    public fun assert_rooch_genesis_address(addr: address) {
        assert!(is_rooch_genesis_address(addr), ErrorNotGenesisAddress)
    }

    public fun is_rooch_genesis_address(addr: address): bool {
        addr == genesis_address()
    }

    public fun assert_rooch_framework(account: &signer) {
        assert!(
            is_rooch_framework_address(signer::address_of(account)),
            ErrorNotRoochFrameworkAddress,
        )
    }

    /// Return true if `addr` is 0x3.
    public fun is_rooch_framework_address(addr: address): bool {
        addr == @rooch_framework
    }

    /// The address of the genesis
    public fun genesis_address(): address {
        @rooch_framework
    }
}
