/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use std::signer;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;

    use rooch_framework::coin::{Self, BurnCapability, MintCapability};
    use rooch_framework::core_addresses;

    friend rooch_framework::genesis;

    /// Account does not have mint capability
    const ENoCapabilities: u64 = 1;

    struct GasCoin has key {}

    struct MintCapStore has key {
        mint_cap: MintCapability<GasCoin>,
    }

    /// Delegation coin created by delegator and can be claimed by the delegatee as MintCapability.
    struct DelegatedMintCapability has store {
        to: address
    }

    /// The container stores the current pending delegations.
    struct Delegations has key {
        inner: vector<DelegatedMintCapability>,
    }

    /// Can only called during genesis to initialize the Rooch coin.
    public(friend) fun initialize(ctx: &mut StorageContext, rooch_framework: &signer): (BurnCapability<GasCoin>, MintCapability<GasCoin>) {
        core_addresses::assert_rooch_framework(rooch_framework);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<GasCoin>(
            ctx,
            rooch_framework,
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            9, // decimals
        );

        // Rooch framework needs mint cap to mint coins to initial validators. This will be revoked once the validators
        // have been initialized.
        account_storage::global_move_to(ctx, rooch_framework, MintCapStore { mint_cap });

        coin::destroy_freeze_cap(freeze_cap);
        (burn_cap, mint_cap)
    }

    public fun has_mint_capability(ctx: &StorageContext, account: &signer): bool {
        account_storage::global_exists<MintCapStore>(ctx, signer::address_of(account))
    }

    /// Only called during genesis to destroy the rooch framework account's mint capability once all initial validators
    /// and accounts have been initialized during genesis.
    public(friend) fun destroy_mint_cap(ctx: &mut StorageContext, rooch_framework: &signer) {
        core_addresses::assert_rooch_framework(rooch_framework);
        let MintCapStore { mint_cap } = account_storage::global_move_from<MintCapStore>(ctx,@rooch_framework);
        coin::destroy_mint_cap(mint_cap);
    }

}
