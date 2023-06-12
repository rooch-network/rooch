/**
 * RoochGovernance represents the on-chain governance of the Rooch network.
 *
 */
module rooch_framework::governance {
    use std::error;
    use rooch_framework::account::{Self, SignerCapability};
    use rooch_framework::core_addresses;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;

    friend rooch_framework::genesis;

    /// Store the SignerCapabilities of accounts under the on-chain governance's control.
    struct GovernanceResponsbility has key {
        cap: SignerCapability
    }

    /// The governance responsbility has been store
    const EGovernanceResponsbilityAlreadyExist: u64 = 1;

    /// Can be called during genesis or by the governance itself.
    /// Stores the signer capability for a given address.
    public fun store_signer_cap(
        ctx: &mut StorageContext,
        rooch_genesis: &signer,
        signer_address: address,
        signer_cap: SignerCapability,
    ) {
        core_addresses::assert_rooch_genesis(rooch_genesis);
        core_addresses::assert_framework_reserved(signer_address);

        assert!(!account_storage::global_exists<GovernanceResponsbility>(ctx, core_addresses::genesis_address()), error::invalid_state(EGovernanceResponsbilityAlreadyExist));
        account_storage::global_move_to<GovernanceResponsbility>(
            ctx,
            rooch_genesis,
            GovernanceResponsbility {
                cap: signer_cap
            }
        );
    }

    public(friend) fun get_governance_signer_cap(ctx: &mut StorageContext): signer {
        let cap = account_storage::global_borrow<GovernanceResponsbility>(ctx, core_addresses::genesis_address());
        account::create_signer_with_capability(&cap.cap)
    }

    /// Initializes the state for Rooch Governance. Can only be called during Genesis with a signer
    /// for the rooch_genesis (0x1) account.
    /// TODO rooch governance
    fun initialize(
        rooch_genesis: &signer,
    ) {
        core_addresses::assert_rooch_genesis(rooch_genesis);
    }
}
