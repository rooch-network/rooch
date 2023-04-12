/// The chain id distinguishes between different chains (e.g., testnet and the main network).
/// One important role is to prevent transactions intended for one chain from being executed on another.
/// This code provides a container for storing a chain id and functions to initialize and get it.
module rooch_framework::chain_id {
    use rooch_framework::core_addresses;
    friend rooch_framework::genesis;

    struct ChainId has key {
        id: u8
    }

    const MAIN_CHAIN_ID: u8 = 1;
    const DEV_CHAIN_ID: u8 = 181;
    const TEST_CHAIN_ID: u8 = 182;

    /// Only called during genesis.
    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, id: u8) {
        core_addresses::assert_rooch_genesis(account);
        move_to(account, ChainId { id })
    }

    #[view]
    /// Return the chain ID of this instance.
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(@rooch_framework).id
    }

    public fun is_dev(): bool acquires ChainId {
        get() == DEV_CHAIN_ID
    }

    public fun is_test(): bool acquires ChainId {
        get() == TEST_CHAIN_ID
    }

    public fun is_main(): bool acquires ChainId {
        get() == MAIN_CHAIN_ID
    }

    #[test_only]
    public fun initialize_for_test(rooch_framework: &signer, id: u8) {
        initialize(rooch_framework, id);
    }

    #[test(rooch_framework = @0x1)]
    fun test_get(rooch_framework: &signer) acquires ChainId {
        initialize_for_test(rooch_framework, 1u8);
        assert!(get() == 1u8, 1);
    }
}
