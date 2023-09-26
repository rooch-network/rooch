/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use std::signer;
    use moveos_std::storage_context::StorageContext;
    use rooch_framework::coin::{Self, Coin};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    //TODO should we allow user to transfer gas coin?
    //If not, we can remove `store` ability from GasCoin.
    struct GasCoin has key, store {}

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
        coin::deposit_extend<GasCoin>(ctx, addr, coin);
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
    public(friend) fun genesis_init(ctx: &mut StorageContext, _genesis_account: &signer){
        coin::register_extend<GasCoin>(
            ctx,
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            9, // decimals
        );
    }


}
