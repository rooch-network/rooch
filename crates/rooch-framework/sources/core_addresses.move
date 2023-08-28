module rooch_framework::core_addresses {
    use std::error;
    use std::signer;

    /// The address/account did not correspond to the genesis address
    const ErrorNotGenesisAddress: u64 = 1;
    /// The address/account did not correspond to the association address
    const ErrorNotAssociationAddress: u64 = 2;
    /// The operation can only be performed by the VM
    const ErrorVm: u64 = 3;
    /// The address/account did not correspond to the core framework address
    const ErrorNotRoochFrameworkAddress: u64 = 4;
    /// The address is not framework reserved address
    const ErrorNotFrameworkReservedAddress: u64 = 5;


    public fun assert_rooch_genesis(account: &signer) {
        assert_rooch_genesis_address(signer::address_of(account))
    }

    public fun assert_rooch_genesis_address(addr: address) {
        assert!(is_rooch_genesis_address(addr), error::permission_denied(ENotGenesisAddress))
    }

    public fun is_rooch_genesis_address(addr: address): bool {
        addr == genesis_address()
    }

    public fun assert_rooch_association(account: &signer) {
        assert_rooch_association_address(signer::address_of(account))
    }

    public fun assert_rooch_association_address(addr: address) {
        assert!(is_rooch_association_address(addr), error::permission_denied(ENotAssociationAddress))
    }

    public fun is_rooch_association_address(addr: address): bool {
        addr == @rooch_association
    }

    public fun assert_rooch_framework(account: &signer) {
        assert!(
            is_rooch_framework_address(signer::address_of(account)),
            error::permission_denied(ENotRoochFrameworkAddress),
        )
    }

    public fun assert_framework_reserved_address(account: &signer) {
        assert_framework_reserved(signer::address_of(account));
    }

    public fun assert_framework_reserved(addr: address) {
        assert!(
            is_framework_reserved_address(addr),
            error::permission_denied(ENotFrameworkReservedAddress),
        )
    }

    /// Return true if `addr` is 0x0 or under the on chain governance's control.
    public fun is_framework_reserved_address(addr: address): bool {
        is_rooch_framework_address(addr) ||
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

    /// Return true if `addr` is 0x3.
    public fun is_rooch_framework_address(addr: address): bool {
        addr == @rooch_framework
    }

    /// Assert that the signer has the VM reserved address.
    public fun assert_vm(account: &signer) {
        assert!(is_vm(account), error::permission_denied(EVm))
    }

    /// Return true if `addr` is a reserved address for the VM to call system modules.
    public fun is_vm(account: &signer): bool {
        is_vm_address(signer::address_of(account))
    }

    /// Return true if `addr` is a reserved address for the VM to call system modules.
    public fun is_vm_address(addr: address): bool {
        addr == @vm_reserved
    }

    /// Return true if `addr` is either the VM address or an Rooch Framework address.
    public fun is_reserved_address(addr: address): bool {
        is_rooch_framework_address(addr) || is_vm_address(addr)
    }

    /// The address of the genesis
    public fun genesis_address(): address {
        @rooch_framework
    }

}
