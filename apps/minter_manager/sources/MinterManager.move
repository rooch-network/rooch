module minter_manager::minter_manager {

    // =========================== Packages ===========================
    use moveos_std::event;
    use moveos_std::signer;
    use moveos_std::table;
    use moveos_std::object::{Self, Object, ObjectID};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;


    // =========================== Constants ==========================
    const ENOT_ADMIN: u64 = 120;
    const ETREASURY_CAP_MANAGER_DESTROYED: u64 = 121;
    const EMINTER_REVOKED: u64 = 122;


    // ============================ Storage ===========================
    struct TreasuryCapManager<phantom CoinType: key + store> has key {
        admin: address,
        coinInfoObj: Object<CoinInfo<CoinType>>,
        revokedMinters: table::Table<ObjectID, bool>,
    }

    struct MinterCap<phantom CoinType: key + store> has key, store {
        managerId: ObjectID,
    }

    #[event]
    struct AdminTransferred has drop, copy {
        prevAdmin: address,
        newAdmin: address,
    }

    #[event]
    struct TreasuryCapManagerSetup has drop, copy {
        admin: address,
        treasuryCapManagerId: ObjectID,
    }

    #[event]
    struct TreasuryCapManagerDestroyed has drop, copy {
        treasuryCapManagerId: ObjectID,
    }

    #[event]
    struct MinterCapIssued has drop, copy {
        recipient: address,
        minterCapId: ObjectID,
    }

    #[event]
    struct MinterCapRevoked has drop, copy {
        minterCapId: ObjectID,
    }

    #[event]
    struct MinterCapDestroyed has drop, copy {
        minterCapId: ObjectID,
    }

    
    // =========================== Coin Admin Functions ===========================
    public entry fun transferAdmin<CoinType: key + store>(
        coinAdmin: &signer,
        treasuryCapManagerObj: &mut Object<TreasuryCapManager<CoinType>>,
        newAdmin: address,
    ) {
        let treasuryCapManager = object::borrow_mut(treasuryCapManagerObj);
        assert!(signer::address_of(coinAdmin) == treasuryCapManager.admin, ENOT_ADMIN);
        treasuryCapManager.admin = newAdmin;
        event::emit(AdminTransferred { prevAdmin: signer::address_of(coinAdmin), newAdmin });
    }

    public entry fun setupTreasuryCapManager<CoinType: key + store>(
        coinAdmin: &signer,
        coinInfoObj: Object<CoinInfo<CoinType>>,
    ) {
        let treasuryCapManager = TreasuryCapManager<CoinType> {
            admin: signer::address_of(coinAdmin),
            coinInfoObj,
            revokedMinters: table::new(),
        };
        let treasuryCapManagerObject = object::new(treasuryCapManager);
        let treasuryCapManagerId = object::id(&treasuryCapManagerObject);
        object::to_shared(treasuryCapManagerObject);
        event::emit(TreasuryCapManagerSetup { admin: signer::address_of(coinAdmin), treasuryCapManagerId });
    }

    public entry fun destroyTreasuryCapManager<CoinType: key + store>(
        coinAdmin: &signer,
        treasuryCapManagerId: ObjectID,
    ) {
        let treasuryCapManager = object::remove(
            object::take_object_extend<TreasuryCapManager<CoinType>>(treasuryCapManagerId)
        );
        assert!(signer::address_of(coinAdmin) == treasuryCapManager.admin, ENOT_ADMIN);
        let TreasuryCapManager<CoinType> {
            admin: _, coinInfoObj, revokedMinters,
        } = treasuryCapManager;

        table::drop(revokedMinters);
        object::transfer(coinInfoObj, signer::address_of(coinAdmin));
        event::emit(TreasuryCapManagerDestroyed { treasuryCapManagerId });
    }

    public entry fun issueMinterCap<CoinType: key + store>(
        coinAdmin: &signer,
        treasuryCapManagerObj: &mut Object<TreasuryCapManager<CoinType>>,
        recipient: address,
    ) {
        let treasuryCapManager = object::borrow_mut(treasuryCapManagerObj);
        assert!(signer::address_of(coinAdmin) == treasuryCapManager.admin, ENOT_ADMIN);
        let minterCapObj = object::new(MinterCap<CoinType> {
            managerId: object::id(treasuryCapManagerObj),
        });
        let minterCapId = object::id(&minterCapObj);
        object::transfer(minterCapObj, recipient);
        event::emit(MinterCapIssued { recipient, minterCapId });
    }

    public entry fun revokeMinterCap<CoinType: key + store>(
        coinAdmin: &signer,
        treasuryCapManagerObj: &mut Object<TreasuryCapManager<CoinType>>,
        minterCapId: ObjectID,
    ) {
        let treasuryCapManager = object::borrow_mut(treasuryCapManagerObj);
        assert!(signer::address_of(coinAdmin) == treasuryCapManager.admin, ENOT_ADMIN);
        table::add(&mut treasuryCapManager.revokedMinters, minterCapId, true);
        event::emit(MinterCapRevoked { minterCapId });
    }

    public entry fun destroyMinterCap<CoinType: key + store>(
        _minter: &signer,
        minterCapObj: Object<MinterCap<CoinType>>,
    ) {
        let minterCapId = object::id(&minterCapObj);
        let MinterCap<CoinType> { managerId: _ } = object::remove(minterCapObj);
        event::emit(MinterCapDestroyed { minterCapId });
    }


    // =========================== Minter Functions ===========================
    public entry fun mint<CoinType: key + store>(
        _minter: &signer,
        treasuryCapManagerObj: &mut Object<TreasuryCapManager<CoinType>>,
        minterCapObj: &mut Object<MinterCap<CoinType>>,
        amount: u256,
        recipient: address,
    ) {
        let treasuryCapManagerId = object::id(treasuryCapManagerObj);
        let treasuryCapManager = object::borrow_mut(treasuryCapManagerObj);
        let minterCapId = object::id(minterCapObj);
        let minterCap = object::borrow(minterCapObj);
        assert!(
            !table::contains(&treasuryCapManager.revokedMinters, minterCapId),
            EMINTER_REVOKED,
        );
        assert!(
            minterCap.managerId == treasuryCapManagerId,
            ETREASURY_CAP_MANAGER_DESTROYED,
        );
        let coinToDeposit = coin::mint(&mut treasuryCapManager.coinInfoObj, amount);
        account_coin_store::deposit(recipient, coinToDeposit);
    }

    public entry fun burnFromSigner<CoinType: key + store>(
        burner: &signer,
        treasuryCapManagerObj: &mut Object<TreasuryCapManager<CoinType>>,
        minterCapObj: &mut Object<MinterCap<CoinType>>,
        amount: u256,
    ) {
        let coinToBurn = account_coin_store::withdraw(burner, amount);
        burn(burner, treasuryCapManagerObj, minterCapObj, coinToBurn);
    }

    public fun burn<CoinType: key + store>(
        _sender: &signer,
        treasuryCapManagerObj: &mut Object<TreasuryCapManager<CoinType>>,
        minterCapObj: &mut Object<MinterCap<CoinType>>,
        coinToBurn: Coin<CoinType>,
    ) {
        let treasuryCapManagerId = object::id(treasuryCapManagerObj);
        let treasuryCapManager = object::borrow_mut(treasuryCapManagerObj);
        let minterCapId = object::id(minterCapObj);
        let minterCap = object::borrow(minterCapObj);
        assert!(
            !table::contains(&treasuryCapManager.revokedMinters, minterCapId),
            EMINTER_REVOKED,
        );
        assert!(
            minterCap.managerId == treasuryCapManagerId,
            ETREASURY_CAP_MANAGER_DESTROYED,
        );
        coin::burn(&mut treasuryCapManager.coinInfoObj, coinToBurn);
    }


    // =========================== Test ===========================

    #[test_only]
    use std::option;

    #[test_only]
    use std::string::utf8;

    #[test_only]
    struct FakeMoney has key, store {}

    #[test(coinAdmin = @minter_manager)]
    fun testIssueCoin(coinAdmin: &signer): ObjectID {
        rooch_framework::genesis::init_for_test();
        let coinInfoObj = coin::register_extend<FakeMoney>(
            utf8(b"FakeMoney"),
            utf8(b"FM"),
            option::none(),
            18,      // decimal
        );
        let coinInfoObjId = object::id(&coinInfoObj);
        object::transfer(coinInfoObj, signer::address_of(coinAdmin));
        coinInfoObjId
    }

    #[test(coinAdmin = @minter_manager, to = @0x44cc)]
    fun testNormalMint(coinAdmin: &signer, to: &signer) {
        let coinInfoObjId = testIssueCoin(coinAdmin);
        let coinInfoObjMut = object::borrow_mut_object<CoinInfo<FakeMoney>>(
            coinAdmin, coinInfoObjId
        );
        let coinToDeposit = coin::mint(coinInfoObjMut, 1000000);
        account_coin_store::deposit(signer::address_of(to), coinToDeposit);

    }

    #[test(coinAdmin = @minter_manager)]
    fun testSetupTreasury(coinAdmin: &signer) {
        let coinInfoObjId = testIssueCoin(coinAdmin);
        let coinInfoObj = object::take_object(coinAdmin, coinInfoObjId);
        setupTreasuryCapManager<FakeMoney>(coinAdmin, coinInfoObj);
    }

}
