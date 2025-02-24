module free_tunnel_rooch::atomic_lock {

    // =========================== Packages ===========================
    use moveos_std::account;
    use moveos_std::event;
    use moveos_std::signer;
    use moveos_std::table;
    use moveos_std::timestamp::now_seconds;
    use moveos_std::object::Object;

    use rooch_framework::account_coin_store;
    use rooch_framework::coin_store::{Self, CoinStore};
    
    use free_tunnel_rooch::req_helpers::{Self, EXPIRE_PERIOD, EXPIRE_EXTRA_PERIOD};
    use free_tunnel_rooch::permissions;


    // =========================== Constants ==========================
    const EXECUTED_PLACEHOLDER: address = @0xed;
    const DEPLOYER: address = @free_tunnel_rooch;

    const ENOT_LOCK_MINT: u64 = 70;
    const EINVALID_REQ_ID: u64 = 71;
    const EINVALID_PROPOSER: u64 = 72;
    const EWAIT_UNTIL_EXPIRED: u64 = 73;
    const ENOT_BURN_UNLOCK: u64 = 74;
    const EINVALID_RECIPIENT: u64 = 75;
    const ENOT_DEPLOYER: u64 = 76;


    // ============================ Storage ===========================
    struct AtomicLockStorage has key, store {
        proposedLock: table::Table<vector<u8>, address>,
        proposedUnlock: table::Table<vector<u8>, address>,
        lockedBalanceOf: table::Table<u8, u256>,
    }

    struct CoinStorage<phantom CoinType: key + store> has key {
        lockedCoins: Object<CoinStore<CoinType>>,
    }

    #[event]
    struct TokenLockProposed has drop, copy {
        reqId: vector<u8>,
        proposer: address,
    }

    #[event]
    struct TokenLockExecuted has drop, copy {
        reqId: vector<u8>,
        proposer: address,
    }

    #[event]
    struct TokenLockCancelled has drop, copy {
        reqId: vector<u8>,
        proposer: address,
    }

    #[event]
    struct TokenUnlockProposed has drop, copy {
        reqId: vector<u8>,
        recipient: address,
    }

    #[event]
    struct TokenUnlockExecuted has drop, copy {
        reqId: vector<u8>,
        recipient: address,
    }

    #[event]
    struct TokenUnlockCancelled has drop, copy {
        reqId: vector<u8>,
        recipient: address,
    }

    fun init(admin: &signer) {
        let atomicLockStorage = AtomicLockStorage {
            proposedLock: table::new(),
            proposedUnlock: table::new(),
            lockedBalanceOf: table::new(),
        };
        account::move_resource_to(admin, atomicLockStorage);
    }


    // =========================== Functions ===========================
    public entry fun addToken<CoinType: key + store>(
        admin: &signer,
        tokenIndex: u8,
        decimals: u8,
    ) {
        permissions::assertOnlyAdmin(admin);
        req_helpers::addTokenInternal<CoinType>(tokenIndex, decimals);
        if (!account::exists_resource<CoinStorage<CoinType>>(@free_tunnel_rooch)) {
            let coinStorage = CoinStorage<CoinType> {
                lockedCoins: coin_store::create_coin_store<CoinType>()
            };
            account::move_resource_to(admin, coinStorage);
        }
    }
    

    public entry fun removeToken<CoinType: key + store>(
        admin: &signer,
        tokenIndex: u8,
    ) {
        permissions::assertOnlyAdmin(admin);
        req_helpers::removeTokenInternal(tokenIndex);
    }


    public entry fun proposeLock<CoinType: key + store>(
        proposer: &signer,
        reqId: vector<u8>,
    ) {
        let storeA = account::borrow_mut_resource<AtomicLockStorage>(@free_tunnel_rooch);
        req_helpers::assertFromChainOnly(&reqId);
        req_helpers::checkCreatedTimeFrom(&reqId);
        let action = req_helpers::actionFrom(&reqId);
        assert!(action & 0x0f == 1, ENOT_LOCK_MINT);
        assert!(!table::contains(&storeA.proposedLock, reqId), EINVALID_REQ_ID);

        let proposerAddress = signer::address_of(proposer);
        assert!(proposerAddress != EXECUTED_PLACEHOLDER, EINVALID_PROPOSER);

        let amount = req_helpers::amountFrom<CoinType>(&reqId);
        let _tokenIndex = req_helpers::tokenIndexFrom<CoinType>(&reqId);
        table::add(&mut storeA.proposedLock, reqId, proposerAddress);

        let coinStorage = account::borrow_mut_resource<CoinStorage<CoinType>>(@free_tunnel_rooch);
        let coinToLock = account_coin_store::withdraw<CoinType>(proposer, amount);
        coin_store::deposit(&mut coinStorage.lockedCoins, coinToLock);
        event::emit(TokenLockProposed{ reqId, proposer: proposerAddress });
    }
    

    public entry fun executeLock<CoinType: key + store>(
        _sender: &signer,
        reqId: vector<u8>,
        r: vector<vector<u8>>,
        yParityAndS: vector<vector<u8>>,
        executors: vector<vector<u8>>,
        exeIndex: u64,
    ) {
        let storeA = account::borrow_mut_resource<AtomicLockStorage>(@free_tunnel_rooch);
        let proposerAddress = *table::borrow(&storeA.proposedLock, reqId);
        assert!(proposerAddress != EXECUTED_PLACEHOLDER, EINVALID_REQ_ID);

        let message = req_helpers::msgFromReqSigningMessage(&reqId);
        permissions::checkMultiSignatures(
            message, r, yParityAndS, executors, exeIndex,
        );

        *table::borrow_mut(&mut storeA.proposedLock, reqId) = EXECUTED_PLACEHOLDER;

        let amount = req_helpers::amountFrom<CoinType>(&reqId);
        let tokenIndex = req_helpers::tokenIndexFrom<CoinType>(&reqId);

        if (table::contains(&storeA.lockedBalanceOf, tokenIndex)) {
            let originalAmount = *table::borrow(&storeA.lockedBalanceOf, tokenIndex);
            *table::borrow_mut(&mut storeA.lockedBalanceOf, tokenIndex) = originalAmount + amount;
        } else {
            table::add(&mut storeA.lockedBalanceOf, tokenIndex, amount);
        };
        event::emit(TokenLockExecuted{ reqId, proposer: proposerAddress });
    }


    public entry fun cancelLock<CoinType: key + store>(
        _sender: &signer,
        reqId: vector<u8>,
    ) {
        let storeA = account::borrow_mut_resource<AtomicLockStorage>(@free_tunnel_rooch);
        let proposerAddress = *table::borrow(&storeA.proposedLock, reqId);
        assert!(proposerAddress != EXECUTED_PLACEHOLDER, EINVALID_REQ_ID);
        assert!(
            now_seconds() > req_helpers::createdTimeFrom(&reqId) + EXPIRE_PERIOD(),
            EWAIT_UNTIL_EXPIRED,
        );
        table::remove(&mut storeA.proposedLock, reqId);

        let amount = req_helpers::amountFrom<CoinType>(&reqId);
        let _tokenIndex = req_helpers::tokenIndexFrom<CoinType>(&reqId);
        
        let coinStorage = account::borrow_mut_resource<CoinStorage<CoinType>>(@free_tunnel_rooch);
        let coinInside = &mut coinStorage.lockedCoins;
        let coinCancelled = coin_store::withdraw(coinInside, amount);

        account_coin_store::deposit(proposerAddress, coinCancelled);
        event::emit(TokenLockCancelled{ reqId, proposer: proposerAddress });
    }


    public entry fun proposeUnlock<CoinType: key + store>(
        proposer: &signer,
        reqId: vector<u8>,
        recipient: address,
    ) {
        let storeA = account::borrow_mut_resource<AtomicLockStorage>(@free_tunnel_rooch);
        permissions::assertOnlyProposer(proposer);
        req_helpers::assertFromChainOnly(&reqId);
        req_helpers::checkCreatedTimeFrom(&reqId);
        assert!(req_helpers::actionFrom(&reqId) & 0x0f == 2, ENOT_BURN_UNLOCK);
        assert!(!table::contains(&storeA.proposedUnlock, reqId), EINVALID_REQ_ID);
        assert!(recipient != EXECUTED_PLACEHOLDER, EINVALID_RECIPIENT);

        let amount = req_helpers::amountFrom<CoinType>(&reqId);
        let tokenIndex = req_helpers::tokenIndexFrom<CoinType>(&reqId);
        let originalAmount = *table::borrow(&storeA.lockedBalanceOf, tokenIndex);
        *table::borrow_mut(&mut storeA.lockedBalanceOf, tokenIndex) = originalAmount - amount;
        table::add(&mut storeA.proposedUnlock, reqId, recipient);
        event::emit(TokenUnlockProposed{ reqId, recipient });
    }


    public entry fun executeUnlock<CoinType: key + store>(
        _sender: &signer,
        reqId: vector<u8>,
        r: vector<vector<u8>>,
        yParityAndS: vector<vector<u8>>,
        executors: vector<vector<u8>>,
        exeIndex: u64,
    ) {
        let storeA = account::borrow_mut_resource<AtomicLockStorage>(@free_tunnel_rooch);
        let recipient = *table::borrow(&storeA.proposedUnlock, reqId);
        assert!(recipient != EXECUTED_PLACEHOLDER, EINVALID_REQ_ID);

        let message = req_helpers::msgFromReqSigningMessage(&reqId);
        permissions::checkMultiSignatures(
            message, r, yParityAndS, executors, exeIndex,
        );

        *table::borrow_mut(&mut storeA.proposedUnlock, reqId) = EXECUTED_PLACEHOLDER;

        let amount = req_helpers::amountFrom<CoinType>(&reqId);
        let _tokenIndex = req_helpers::tokenIndexFrom<CoinType>(&reqId);

        let coinStorage = account::borrow_mut_resource<CoinStorage<CoinType>>(@free_tunnel_rooch);
        let coinInside = &mut coinStorage.lockedCoins;
        let coinUnlocked = coin_store::withdraw(coinInside, amount);

        account_coin_store::deposit(recipient, coinUnlocked);
        event::emit(TokenUnlockExecuted{ reqId, recipient });
    }


    public entry fun cancelUnlock<CoinType: key + store>(
        _sender: &signer,
        reqId: vector<u8>,
    ) {
        let storeA = account::borrow_mut_resource<AtomicLockStorage>(@free_tunnel_rooch);
        let recipient = *table::borrow(&storeA.proposedUnlock, reqId);
        assert!(recipient != EXECUTED_PLACEHOLDER, EINVALID_REQ_ID);
        assert!(
            now_seconds() > req_helpers::createdTimeFrom(&reqId) + EXPIRE_EXTRA_PERIOD(),
            EWAIT_UNTIL_EXPIRED,
        );

        table::remove(&mut storeA.proposedUnlock, reqId);
        let amount = req_helpers::amountFrom<CoinType>(&reqId);
        let tokenIndex = req_helpers::tokenIndexFrom<CoinType>(&reqId);
        let originalAmount = *table::borrow(&storeA.lockedBalanceOf, tokenIndex);
        *table::borrow_mut(&mut storeA.lockedBalanceOf, tokenIndex) = originalAmount + amount;
        event::emit(TokenUnlockCancelled{ reqId, recipient });
    }

}