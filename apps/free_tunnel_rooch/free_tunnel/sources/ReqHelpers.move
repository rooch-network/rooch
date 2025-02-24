module free_tunnel_rooch::req_helpers {

    // =========================== Packages ===========================
    use std::u64;
    use std::vector;
    use moveos_std::account;
    use moveos_std::event;
    use moveos_std::table;
    use moveos_std::timestamp::now_seconds;
    use moveos_std::type_info::{Self, TypeInfo};
    use free_tunnel_rooch::utils::{smallU64ToString, hexToString};
    friend free_tunnel_rooch::permissions;
    friend free_tunnel_rooch::atomic_mint;
    friend free_tunnel_rooch::atomic_lock;
    

    // =========================== Constants ==========================
    const CHAIN: u8 = 0xb0;

    const ETOKEN_INDEX_OCCUPIED: u64 = 0;
    const ETOKEN_INDEX_CANNOT_BE_ZERO: u64 = 1;
    const ETOKEN_INDEX_NONEXISTENT: u64 = 2;
    const EINVALID_REQ_ID_LENGTH: u64 = 3;
    const ENOT_FROM_CURRENT_CHAIN: u64 = 4;
    const ENOT_TO_CURRENT_CHAIN: u64 = 5;
    const ECREATED_TIME_TOO_EARLY: u64 = 6;
    const ECREATED_TIME_TOO_LATE: u64 = 7;
    const EAMOUNT_CANNOT_BE_ZERO: u64 = 8;
    const ETOKEN_TYPE_MISMATCH: u64 = 9;

    public(friend) fun BRIDGE_CHANNEL(): vector<u8> { b"SolvBTC Bridge" }
    public(friend) fun PROPOSE_PERIOD(): u64 { 172800 }         // 48 hours
    public(friend) fun EXPIRE_PERIOD(): u64 { 259200 }          // 72 hours
    public(friend) fun EXPIRE_EXTRA_PERIOD(): u64 { 345600 }    // 96 hours
    public(friend) fun ETH_SIGN_HEADER(): vector<u8> { b"\x19Ethereum Signed Message:\n" }


    // ============================ Storage ===========================
    struct ReqHelpersStorage has key {
        tokens: table::Table<u8, TypeInfo>,
        tokenDecimals: table::Table<u8, u8>,
    }

    fun init(admin: &signer) {
        initReqHelpersStorage(admin);
    }

    public(friend) fun initReqHelpersStorage(admin: &signer) {
        account::move_resource_to(admin, ReqHelpersStorage {
            tokens: table::new(),
            tokenDecimals: table::new(),
        })
    }

    #[event]
    struct TokenAdded has drop, copy {
        tokenIndex: u8,
        tokenType: TypeInfo,
    }
    
    #[event]
    struct TokenRemoved has drop, copy {
        tokenIndex: u8,
        tokenType: TypeInfo,
    }


    // =========================== Functions ===========================
    public(friend) fun addTokenInternal<CoinType: key>(tokenIndex: u8, decimals: u8) {
        let storeR = account::borrow_mut_resource<ReqHelpersStorage>(@free_tunnel_rooch);

        assert!(
            !table::contains(&storeR.tokens, tokenIndex), 
            ETOKEN_INDEX_OCCUPIED
        );
        assert!(tokenIndex > 0, ETOKEN_INDEX_CANNOT_BE_ZERO);

        let tokenType = type_info::type_of<CoinType>();
        table::add(&mut storeR.tokens, tokenIndex, tokenType);
        table::add(&mut storeR.tokenDecimals, tokenIndex, decimals);
        event::emit(TokenAdded { tokenIndex, tokenType });
    }

    public(friend) fun removeTokenInternal(tokenIndex: u8) {
        let storeR = account::borrow_mut_resource<ReqHelpersStorage>(@free_tunnel_rooch);
        assert!(table::contains(&storeR.tokens, tokenIndex), ETOKEN_INDEX_NONEXISTENT);
        assert!(tokenIndex > 0, ETOKEN_INDEX_CANNOT_BE_ZERO);
        let tokenType = table::remove(&mut storeR.tokens, tokenIndex);
        table::remove(&mut storeR.tokenDecimals, tokenIndex);
        event::emit(TokenRemoved { tokenIndex, tokenType });
    }

    /// `reqId` in format of `version:uint8|createdTime:uint40|action:uint8|tokenIndex:uint8|amount:uint64|from:uint8|to:uint8|(TBD):uint112`
    public(friend) fun versionFrom(reqId: &vector<u8>): u8 {
        *vector::borrow(reqId, 0)
    }

    public(friend) fun createdTimeFrom(reqId: &vector<u8>): u64 {
        let time = (*vector::borrow(reqId, 1) as u64);
        let i = 2;
        while (i < 6) {
            time = (time << 8) + (*vector::borrow(reqId, i) as u64);
            i = i + 1;
        };
        time
    }

    public(friend) fun checkCreatedTimeFrom(reqId: &vector<u8>): u64 {
        let time = createdTimeFrom(reqId);
        assert!(time > now_seconds() - PROPOSE_PERIOD(), ECREATED_TIME_TOO_EARLY);
        assert!(time < now_seconds() + 60, ECREATED_TIME_TOO_LATE);
        time
    }

    public(friend) fun actionFrom(reqId: &vector<u8>): u8 {
        *vector::borrow(reqId, 6)
    }

    public(friend) fun decodeTokenIndex(reqId: &vector<u8>): u8 {
        *vector::borrow(reqId, 7)
    }

    public(friend) fun checkTokenType<CoinType>(tokenIndex: u8) {
        let storeR = account::borrow_mut_resource<ReqHelpersStorage>(@free_tunnel_rooch);
        let tokenTypeExpected = *table::borrow(&storeR.tokens, tokenIndex);
        let tokenTypeActual = type_info::type_of<CoinType>();
        assert!(tokenTypeExpected == tokenTypeActual, ETOKEN_TYPE_MISMATCH);
    }

    public(friend) fun tokenIndexFrom<CoinType>(reqId: &vector<u8>): u8 {
        let tokenIndex = decodeTokenIndex(reqId);
        let storeR = account::borrow_mut_resource<ReqHelpersStorage>(@free_tunnel_rooch);
        assert!(table::contains(&storeR.tokens, tokenIndex), ETOKEN_INDEX_NONEXISTENT);
        checkTokenType<CoinType>(tokenIndex);
        tokenIndex
    }

    fun decodeAmount(reqId: &vector<u8>): u64 {
        let amount = (*vector::borrow(reqId, 8) as u64);
        let i = 9;
        while (i < 16) {
            amount = (amount << 8) + (*vector::borrow(reqId, i) as u64);
            i = i + 1;
        };
        assert!(amount > 0, EAMOUNT_CANNOT_BE_ZERO);
        amount
    }

    public(friend) fun amountFrom<CoinType>(reqId: &vector<u8>): u256 {
        let storeR = account::borrow_mut_resource<ReqHelpersStorage>(@free_tunnel_rooch);
        let amount = decodeAmount(reqId);
        let tokenIndex = decodeTokenIndex(reqId);
        let decimals = *table::borrow(&storeR.tokenDecimals, tokenIndex);
        if (decimals > 6) {
            amount = amount * u64::pow(10, decimals - 6);
        } else if (decimals < 6) {
            amount = amount / u64::pow(10, 6 - decimals);
        };
        (amount as u256)
    }

    public(friend) fun msgFromReqSigningMessage(reqId: &vector<u8>): vector<u8> {
        assert!(vector::length(reqId) == 32, EINVALID_REQ_ID_LENGTH);
        let specificAction = actionFrom(reqId) & 0x0f;
        if (specificAction == 1) {
            let msg = ETH_SIGN_HEADER();
            vector::append(&mut msg, smallU64ToString(3 + vector::length(&BRIDGE_CHANNEL()) + 29 + 66));
            vector::append(&mut msg, b"[");
            vector::append(&mut msg, BRIDGE_CHANNEL());
            vector::append(&mut msg, b"]\n");
            vector::append(&mut msg, b"Sign to execute a lock-mint:\n");
            vector::append(&mut msg, hexToString(reqId, true));
            msg
        } else if (specificAction == 2) {
            let msg = ETH_SIGN_HEADER();
            vector::append(&mut msg, smallU64ToString(3 + vector::length(&BRIDGE_CHANNEL()) + 31 + 66));
            vector::append(&mut msg, b"[");
            vector::append(&mut msg, BRIDGE_CHANNEL());
            vector::append(&mut msg, b"]\n");
            vector::append(&mut msg, b"Sign to execute a burn-unlock:\n");
            vector::append(&mut msg, hexToString(reqId, true));
            msg
        } else if (specificAction == 3) {
            let msg = ETH_SIGN_HEADER();
            vector::append(&mut msg, smallU64ToString(3 + vector::length(&BRIDGE_CHANNEL()) + 29 + 66));
            vector::append(&mut msg, b"[");
            vector::append(&mut msg, BRIDGE_CHANNEL());
            vector::append(&mut msg, b"]\n");
            vector::append(&mut msg, b"Sign to execute a burn-mint:\n");
            vector::append(&mut msg, hexToString(reqId, true));
            msg
        } else {
            vector::empty<u8>()
        }
    }

    public(friend) fun assertFromChainOnly(reqId: &vector<u8>) {
        assert!(CHAIN == *vector::borrow(reqId, 16), ENOT_FROM_CURRENT_CHAIN);
    }

    public(friend) fun assertToChainOnly(reqId: &vector<u8>) {
        assert!(CHAIN == *vector::borrow(reqId, 17), ENOT_TO_CURRENT_CHAIN);
    }

    #[test]
    fun testDecodingReqid() {
        // `version:uint8|createdTime:uint40|action:uint8|tokenIndex:uint8|amount:uint64|from:uint8|to:uint8|(TBD):uint112`
        let reqId = x"112233445566778899aabbccddeeff00b0b0ffffffffffffffffffffffffffff";
        assert!(versionFrom(&reqId) == 0x11, 1);
        assert!(createdTimeFrom(&reqId) == 0x2233445566, 1);
        assert!(actionFrom(&reqId) == 0x77, 1);
        assert!(decodeTokenIndex(&reqId) == 0x88, 1);
        assert!(decodeAmount(&reqId) == 0x99aabbccddeeff00, 1);
        assertFromChainOnly(&reqId);
        assertToChainOnly(&reqId);
    }

    #[test]
    fun testMsgFromReqSigningMessage1() {
        // action 1: lock-mint
        let reqId = x"112233445566018899aabbccddeeff004040ffffffffffffffffffffffffffff";
        let expected = b"\x19Ethereum Signed Message:\n112[SolvBTC Bridge]\nSign to execute a lock-mint:\n0x112233445566018899aabbccddeeff004040ffffffffffffffffffffffffffff";
        assert!(msgFromReqSigningMessage(&reqId) == expected, 1);
    }

    #[test]
    fun testMsgFromReqSigningMessage2() {
        // action 2: burn-unlock
        let reqId = x"112233445566028899aabbccddeeff004040ffffffffffffffffffffffffffff";
        let expected = b"\x19Ethereum Signed Message:\n114[SolvBTC Bridge]\nSign to execute a burn-unlock:\n0x112233445566028899aabbccddeeff004040ffffffffffffffffffffffffffff";
        assert!(msgFromReqSigningMessage(&reqId) == expected, 1);
    }

    #[test]
    fun testMsgFromReqSigningMessage3() {
        // action 3: burn-mint
        let reqId = x"112233445566038899aabbccddeeff004040ffffffffffffffffffffffffffff";
        let expected = b"\x19Ethereum Signed Message:\n112[SolvBTC Bridge]\nSign to execute a burn-mint:\n0x112233445566038899aabbccddeeff004040ffffffffffffffffffffffffffff";
        assert!(msgFromReqSigningMessage(&reqId) == expected, 1);
    }

    #[test]
    fun testMsgFromReqSigningMessage4() {
        let reqId = x"112233445566048899aabbccddeeff004040ffffffffffffffffffffffffffff";
        assert!(msgFromReqSigningMessage(&reqId) == vector::empty<u8>(), 1);
    }

}