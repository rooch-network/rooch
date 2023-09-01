/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use std::signer;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    use rooch_framework::coin::{Self, Coin, MintCapability};
    use rooch_framework::core_addresses;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    /// Account does not have mint capability
    const ErrorNoCapabilities: u64 = 1;

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

    public fun balance(ctx: &StorageContext, addr: address): u256 {
        coin::balance<GasCoin>(ctx, addr)
    }

    fun mint(ctx: &mut StorageContext, amount: u256): Coin<GasCoin> {
        coin::mint_extend<GasCoin>(ctx, amount)
    }

    #[test_only]
    public fun mint_for_test(ctx: &mut StorageContext, amount: u256) : Coin<GasCoin> {
        mint(ctx, amount)
    }

    public fun burn(ctx: &mut StorageContext, coin: Coin<GasCoin>) {
        coin::burn_extend<GasCoin>(ctx, coin);
    }

    /// deduct gas coin from the given account.
    public(friend) fun deduct_gas(ctx: &mut StorageContext, addr: address, amount: u256):Coin<GasCoin> {
        coin::withdraw_extend<GasCoin>(ctx, addr, amount)
    }

    /// Mint gas coin to the given account.
    public(friend) fun faucet(ctx: &mut StorageContext, addr: address, amount: u256) {
        let coin = mint(ctx, amount);
        coin::deposit<GasCoin>(ctx, addr, coin);
    }

    #[test_only]
    public fun faucet_for_test(ctx: &mut StorageContext, addr: address, amount: u256) {
        faucet(ctx, addr, amount);
    }

    /// TODO find a way to protect this function from DOS attack.
    public entry fun faucet_entry(ctx: &mut StorageContext, account: &signer) {
        let amount = 1_0000_0000u256;
        let addr = signer::address_of(account);
        faucet(ctx, addr, amount);
    }

    /// Can only called during genesis to initialize the Rooch coin.
    public(friend) fun genesis_init(ctx: &mut StorageContext, genesis_account: &signer){
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<GasCoin>(
            ctx,
            genesis_account,
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            9, // decimals
        );

        // Rooch framework needs mint cap to mint coins to initial validators. This will be revoked once the validators
        // have been initialized.
        account_storage::global_move_to(ctx, genesis_account, MintCapStore { mint_cap });

        //TODO do we need the cap?
        coin::destroy_freeze_cap(freeze_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);
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
