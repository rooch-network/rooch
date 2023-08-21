/// CoinEntry is built to make a simple walkthrough of the Coins module.
/// It contains scripts you will need to initialize, mint, burn, transfer coins.
/// By utilizing this current module, a developer can create his own coin and care less about mint and burn capabilities,
module rooch_framework::coin_entry {
    use std::error;
    use std::signer;
    use std::string;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    use rooch_framework::coin::{BurnCapability, MintCapability, FreezeCapability};

    use rooch_framework::coin;
    use rooch_framework::account;

    //
    // Errors
    //

    /// account has no capabilities (burn/mint).
    const ENoCapabilities: u64 = 1;

    //
    // Data structures
    //

    /// Capabilities resource storing mint and burn capabilities.
    /// The resource is stored on the account that initialized coin `CoinType`.
    struct Capabilities<phantom CoinType> has key {
        burn_cap: BurnCapability<CoinType>,
        freeze_cap: FreezeCapability<CoinType>,
        mint_cap: MintCapability<CoinType>,
    }

    //
    // Public functions
    //

    /// Initialize new coin `CoinType` in Rooch Blockchain.
    /// Mint and Burn Capabilities will be stored under `account` in `Capabilities` resource.
    public entry fun initialize<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
    ) {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<CoinType>(
            ctx,
            account,
            string::utf8(name),
            string::utf8(symbol),
            decimals,
        );

        account_storage::global_move_to(ctx, account, Capabilities<CoinType> {
            burn_cap,
            freeze_cap,
            mint_cap
        });
    }

    /// Create new coins `CoinType` and deposit them into dst_addr's account.
    public entry fun mint<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        dst_addr: address,
        amount: u256,
    ) {
        let account_addr = signer::address_of(account);

        assert!(
            // exists<Capabilities<CoinType>>(account_addr),
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ENoCapabilities),
        );

        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        let coins_minted = coin::mint(ctx, amount, &cap.mint_cap);
        account::deposit(ctx, dst_addr, coins_minted);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap)
    }

    /// Withdraw an `amount` of coin `CoinType` from `account` and burn it.
    public entry fun burn<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        amount: u256,
    ) {
        let account_addr = signer::address_of(account);

        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ENoCapabilities),
        );

        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        let to_burn = account::withdraw<CoinType>(ctx, account, amount);
        // let burn_cap = borrow_burn_cap<CoinType>(ctx, account_addr);
        coin::burn<CoinType>(ctx, to_burn, &cap.burn_cap);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap);
    }

    /// Creating a resource that stores balance of `CoinType` on user's account.
    /// Required if user wants to start accepting deposits of `CoinType` in his account.
    public entry fun accept_coin<CoinType>(ctx: &mut StorageContext, account: &signer) {
        account::do_accept_coin<CoinType>(ctx, account)
    }

    /// Enable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun enable_auto_accept_coin(ctx: &mut StorageContext, account: &signer) {
        account::set_auto_accept_coin(ctx, account, true)
    }

    /// Disable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun disable_auto_accept_coin(ctx: &mut StorageContext, account: &signer) {
        account::set_auto_accept_coin(ctx, account, false);
    }

    // /// Deposit the coin balance into the recipient's account and emit an event.
    // public entry fun deposit<CoinType>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
    //     account::deposit<CoinType>(ctx, addr, coin)
    // }

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    public entry fun transfer<CoinType>(
        ctx: &mut StorageContext,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        account::transfer<CoinType>(ctx, from, to, amount)
    }

    /// Freeze a CoinStore to prevent transfers
    public entry fun freeze_coin_store<CoinType>(
        ctx: &mut StorageContext,
        account: &signer
    ) {
        let account_addr = signer::address_of(account);
        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ENoCapabilities),
        );
        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        coin::freeze_coin_store(ctx, account_addr, &cap.freeze_cap);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap)
    }

    /// Unfreeze a CoinStore to allow transfers
    public entry fun unfreeze_coin_store<CoinType>(
        ctx: &mut StorageContext,
        account: &signer
    ) {
        let account_addr = signer::address_of(account);
        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ENoCapabilities),
        );
        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        coin::unfreeze_coin_store(ctx, account_addr, &cap.freeze_cap);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap)
    }

    //
    // Tests
    //

    #[test_only]
    use moveos_std::storage_context;

    #[test_only]
    struct FakeCoin {}

    #[test(source = @0xa11ce, destination = @0xb0b, mod_account = @rooch_framework)]
    public entry fun test_end_to_end(
        source: &signer,
        destination: &signer,
        mod_account: &signer
    ) {
        let source_addr = signer::address_of(source);
        let destination_addr = signer::address_of(destination);

        let source_ctx = storage_context::new_test_context(signer::address_of(source));
        let destination_ctx = storage_context::new_test_context(signer::address_of(destination));
        let mod_account_ctx = storage_context::new_test_context(signer::address_of(mod_account));

        initialize<FakeCoin>(
            &mut mod_account_ctx,
            mod_account,
            b"Fake Coin",
            b"FCD",
            9,
        );
        assert!(coin::is_coin_initialized<FakeCoin>(&source_ctx), 0);

        accept_coin<FakeCoin>(&mut mod_account_ctx, mod_account);
        accept_coin<FakeCoin>(&mut source_ctx, source);
        accept_coin<FakeCoin>(&mut destination_ctx, destination);

        mint<FakeCoin>(&mut mod_account_ctx, mod_account, source_addr, 50);
        mint<FakeCoin>(&mut mod_account_ctx, mod_account, destination_addr, 10);
        assert!(coin::balance<FakeCoin>(&source_ctx, source_addr) == 50, 1);
        assert!(coin::balance<FakeCoin>(&destination_ctx, destination_addr) == 10, 2);

        let supply = coin::supply<FakeCoin>(&mod_account_ctx);
        assert!(supply == 60, 3);

        transfer<FakeCoin>(&mut source_ctx, source, destination_addr, 10);
        assert!(coin::balance<FakeCoin>(&source_ctx, source_addr) == 40, 4);
        assert!(coin::balance<FakeCoin>(&destination_ctx, destination_addr) == 20, 5);

        transfer<FakeCoin>(&mut source_ctx, source, signer::address_of(mod_account), 40);
        burn<FakeCoin>(&mut mod_account_ctx, mod_account, 40);

        assert!(coin::balance<FakeCoin>(&source_ctx, source_addr) == 0, 1);

        let new_supply = coin::supply<FakeCoin>(&source_ctx);
        assert!(new_supply == 20, 2);

        moveos_std::storage_context::drop_test_context(source_ctx);
        moveos_std::storage_context::drop_test_context(destination_ctx);
        moveos_std::storage_context::drop_test_context(mod_account_ctx);
    }

    #[test(source = @0xa11ce, destination = @0xb0b, mod_account = @rooch_framework)]
    #[expected_failure(abort_code = 0x60001, location = Self)]
    public entry fun fail_mint(
        source: &signer,
        destination: &signer,
        mod_account: &signer,
    ) {
        let source_addr = signer::address_of(source);

        let source_ctx = storage_context::new_test_context(signer::address_of(source));
        let destination_ctx = storage_context::new_test_context(signer::address_of(destination));
        let mod_account_ctx = storage_context::new_test_context(signer::address_of(mod_account));

        initialize<FakeCoin>(&mut mod_account_ctx, mod_account, b"Fake Coin", b"FCD", 9);
        accept_coin<FakeCoin>(&mut mod_account_ctx, mod_account);
        accept_coin<FakeCoin>(&mut source_ctx, source);
        accept_coin<FakeCoin>(&mut destination_ctx, destination);

        mint<FakeCoin>(&mut destination_ctx, destination, source_addr, 100);

        moveos_std::storage_context::drop_test_context(source_ctx);
        moveos_std::storage_context::drop_test_context(destination_ctx);
        moveos_std::storage_context::drop_test_context(mod_account_ctx);
    }

    #[test(source = @0xa11ce, destination = @0xb0b, mod_account = @rooch_framework)]
    #[expected_failure(abort_code = 393217, location = Self)]
    public entry fun fail_burn(
        source: &signer,
        destination: &signer,
        mod_account: &signer,
    ) {
        let source_addr = signer::address_of(source);

        let source_ctx = storage_context::new_test_context(signer::address_of(source));
        let destination_ctx = storage_context::new_test_context(signer::address_of(destination));
        let mod_account_ctx = storage_context::new_test_context(signer::address_of(mod_account));

        initialize<FakeCoin>(&mut mod_account_ctx, mod_account, b"Fake Coin", b"FCD", 9);
        accept_coin<FakeCoin>(&mut mod_account_ctx, mod_account);
        accept_coin<FakeCoin>(&mut source_ctx, source);
        accept_coin<FakeCoin>(&mut destination_ctx, destination);

        mint<FakeCoin>(&mut mod_account_ctx, mod_account, source_addr, 100);
        burn<FakeCoin>(&mut destination_ctx, destination, 10);

        moveos_std::storage_context::drop_test_context(source_ctx);
        moveos_std::storage_context::drop_test_context(destination_ctx);
        moveos_std::storage_context::drop_test_context(mod_account_ctx);
    }
}
