module free_tunnel_rooch::permissions {

    // =========================== Packages ===========================
    use std::vector;
    use moveos_std::account;
    use moveos_std::event;
    use moveos_std::signer;
    use moveos_std::table;
    use moveos_std::timestamp::now_seconds;
    use free_tunnel_rooch::utils::{recoverEthAddress, smallU64ToString, smallU64Log10, assertEthAddressList, hexToString};
    use free_tunnel_rooch::req_helpers::{BRIDGE_CHANNEL, ETH_SIGN_HEADER};
    friend free_tunnel_rooch::atomic_mint;
    friend free_tunnel_rooch::atomic_lock;


    // =========================== Constants ==========================
    const ETH_ZERO_ADDRESS: vector<u8> = vector[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    const ENOT_ADMIN: u64 = 20;
    const ENOT_PROPOSER: u64 = 21;
    const EALREADY_PROPOSER: u64 = 22;
    const ENOT_EXISTING_PROPOSER: u64 = 23;
    const EEXECUTORS_ALREADY_INITIALIZED: u64 = 24;
    const ETHRESHOLD_MUST_BE_GREATER_THAN_ZERO: u64 = 25;
    const EARRAY_LENGTH_NOT_EQUAL: u64 = 26;
    const ENOT_MEET_THRESHOLD: u64 = 27;
    const EEXECUTORS_NOT_YET_ACTIVE: u64 = 28;
    const EEXECUTORS_OF_NEXT_INDEX_IS_ACTIVE: u64 = 29;
    const EDUPLICATED_EXECUTORS: u64 = 30;
    const ENON_EXECUTOR: u64 = 31;
    const ESIGNER_CANNOT_BE_EMPTY_ADDRESS: u64 = 32;
    const EINVALID_LENGTH: u64 = 33;
    const EINVALID_SIGNATURE: u64 = 34;
    const EACTIVE_SINCE_SHOULD_AFTER_36H: u64 = 35;
    const EACTIVE_SINCE_SHOULD_WITHIN_5D: u64 = 36;
    const EFAILED_TO_OVERWRITE_EXISTING_EXECUTORS: u64 = 37;


    // ============================ Storage ===========================
    struct PermissionsStorage has key {
        _admin: address,
        _proposerIndex: table::Table<address, u64>,
        _proposerList: vector<address>,
        _executorsForIndex: vector<vector<vector<u8>>>,
        _exeThresholdForIndex: vector<u64>,
        _exeActiveSinceForIndex: vector<u64>,
    }

    fun init(admin: &signer) {
        initPermissionsStorage(admin);
    }

    public(friend) fun initPermissionsStorage(admin: &signer) {
        account::move_resource_to(admin, PermissionsStorage {
            _admin: signer::address_of(admin),
            _proposerIndex: table::new(),
            _proposerList: vector::empty(),
            _executorsForIndex: vector::empty(),
            _exeThresholdForIndex: vector::empty(),
            _exeActiveSinceForIndex: vector::empty(),
        })
    }

    public entry fun initExecutors(
        sender: &signer,
        executors: vector<vector<u8>>,
        threshold: u64,
    ) {
        assertOnlyAdmin(sender);
        initExecutorsInternal(executors, threshold);
    }

    #[event]
    struct AdminTransferred has drop, copy {
        prevAdmin: address,
        newAdmin: address,
    }

    #[event]
    struct ProposerAdded has drop, copy {
        proposer: address,
    }

    #[event]
    struct ProposerRemoved has drop, copy {
        proposer: address,
    }

    #[event]
    struct ExecutorsUpdated has drop, copy {
        executors: vector<vector<u8>>,
        threshold: u64,
        activeSince: u64,
        exeIndex: u64,
    }


    // =========================== Functions ===========================
    public(friend) fun assertOnlyAdmin(sender: &signer) {
        let storeP = account::borrow_resource<PermissionsStorage>(@free_tunnel_rooch);
        assert!(signer::address_of(sender) == storeP._admin, ENOT_ADMIN);
    }

    public(friend) fun assertOnlyProposer(sender: &signer) {
        let storeP = account::borrow_resource<PermissionsStorage>(@free_tunnel_rooch);
        assert!(table::contains(
            &storeP._proposerIndex, 
            signer::address_of(sender)
        ), ENOT_PROPOSER);
    }

    public(friend) fun initAdminInternal(admin: address) {
        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        storeP._admin = admin;
        event::emit(AdminTransferred { prevAdmin: @0x0, newAdmin: admin });
    }

    public(friend) fun transferAdmin(sender: &signer, newAdmin: address) {
        assertOnlyAdmin(sender);
        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        let prevAdmin = storeP._admin;
        storeP._admin = newAdmin;
        event::emit(AdminTransferred { prevAdmin, newAdmin });
    }

    public entry fun addProposer(sender: &signer, proposer: address) {
        assertOnlyAdmin(sender);
        addProposerInternal(proposer);
    }

    public(friend) fun addProposerInternal(proposer: address) {
        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        assert!(!table::contains(&storeP._proposerIndex, proposer), EALREADY_PROPOSER);
        vector::push_back(&mut storeP._proposerList, proposer);
        table::add(&mut storeP._proposerIndex, proposer, vector::length(&storeP._proposerList));
        event::emit(ProposerAdded { proposer });
    }

    public entry fun removeProposer(sender: &signer, proposer: address) {
        assertOnlyAdmin(sender);
        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        assert!(table::contains(&storeP._proposerIndex, proposer), ENOT_EXISTING_PROPOSER);
        let index = table::remove(&mut storeP._proposerIndex, proposer);
        let len = vector::length(&storeP._proposerList);
        if (index < len) {
            let lastProposer = *vector::borrow(&storeP._proposerList, len - 1);
            *vector::borrow_mut(&mut storeP._proposerList, index - 1) = lastProposer;
            *table::borrow_mut(&mut storeP._proposerIndex, lastProposer) = index;
        };
        vector::pop_back(&mut storeP._proposerList);
        event::emit(ProposerRemoved { proposer });
    }

    public(friend) fun initExecutorsInternal(executors: vector<vector<u8>>, threshold: u64) {
        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        assertEthAddressList(&executors);
        assert!(threshold <= vector::length(&executors), ENOT_MEET_THRESHOLD);
        assert!(vector::length(&storeP._exeActiveSinceForIndex) == 0, EEXECUTORS_ALREADY_INITIALIZED);
        assert!(threshold > 0, ETHRESHOLD_MUST_BE_GREATER_THAN_ZERO);
        checkExecutorsNotDuplicated(executors);
        vector::push_back(&mut storeP._executorsForIndex, executors);
        vector::push_back(&mut storeP._exeThresholdForIndex, threshold);
        vector::push_back(&mut storeP._exeActiveSinceForIndex, 1);
        event::emit(ExecutorsUpdated { executors, threshold, activeSince: 1, exeIndex: 0 });
    }

    public entry fun updateExecutors(
        _sender: &signer,
        newExecutors: vector<vector<u8>>,
        threshold: u64,
        activeSince: u64,
        r: vector<vector<u8>>,
        yParityAndS: vector<vector<u8>>,
        executors: vector<vector<u8>>,
        exeIndex: u64,
    ) {
        assertEthAddressList(&newExecutors);
        assert!(threshold > 0, ETHRESHOLD_MUST_BE_GREATER_THAN_ZERO);
        assert!(threshold <= vector::length(&newExecutors), ENOT_MEET_THRESHOLD);
        assert!(
            activeSince > now_seconds() + 36 * 3600,  // 36 hours
            EACTIVE_SINCE_SHOULD_AFTER_36H,
        );
        assert!(
            activeSince < now_seconds() + 120 * 3600,  // 5 days
            EACTIVE_SINCE_SHOULD_WITHIN_5D,
        );
        checkExecutorsNotDuplicated(newExecutors);

        let msg = vector::empty<u8>();
        vector::append(&mut msg, ETH_SIGN_HEADER());
        vector::append(&mut msg, smallU64ToString(
            3 + vector::length(&BRIDGE_CHANNEL()) + (29 + 43 * vector::length(&newExecutors)) 
            + (12 + smallU64Log10(threshold) + 1) + (15 + 10) + (25 + smallU64Log10(exeIndex) + 1)
        ));
        vector::append(&mut msg, b"[");
        vector::append(&mut msg, BRIDGE_CHANNEL());
        vector::append(&mut msg, b"]\n");
        vector::append(&mut msg, b"Sign to update executors to:\n");
        vector::append(&mut msg, joinAddressList(&newExecutors));
        vector::append(&mut msg, b"Threshold: ");
        vector::append(&mut msg, smallU64ToString(threshold));
        vector::append(&mut msg, b"\n");
        vector::append(&mut msg, b"Active since: ");
        vector::append(&mut msg, smallU64ToString(activeSince));
        vector::append(&mut msg, b"\n");
        vector::append(&mut msg, b"Current executors index: ");
        vector::append(&mut msg, smallU64ToString(exeIndex));

        checkMultiSignatures(msg, r, yParityAndS, executors, exeIndex);

        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        let newIndex = exeIndex + 1;
        if (newIndex == vector::length(&storeP._exeActiveSinceForIndex)) {
            vector::push_back(&mut storeP._executorsForIndex, newExecutors);
            vector::push_back(&mut storeP._exeThresholdForIndex, threshold);
            vector::push_back(&mut storeP._exeActiveSinceForIndex, activeSince);
        } else {
            assert!(
                activeSince >= *vector::borrow(&storeP._exeActiveSinceForIndex, newIndex), 
                EFAILED_TO_OVERWRITE_EXISTING_EXECUTORS
            );
            assert!(
                threshold >= *vector::borrow(&storeP._exeThresholdForIndex, newIndex), 
                EFAILED_TO_OVERWRITE_EXISTING_EXECUTORS
            );
            assert!(
                cmpAddrList(newExecutors, *vector::borrow(&storeP._executorsForIndex, newIndex)), 
                EFAILED_TO_OVERWRITE_EXISTING_EXECUTORS
            );
            *vector::borrow_mut(&mut storeP._executorsForIndex, newIndex) = newExecutors;
            *vector::borrow_mut(&mut storeP._exeThresholdForIndex, newIndex) = threshold;
            *vector::borrow_mut(&mut storeP._exeActiveSinceForIndex, newIndex) = activeSince;
        };
        event::emit(ExecutorsUpdated { executors: newExecutors, threshold, activeSince, exeIndex: newIndex });
    }


    fun joinAddressList(ethAddrs: &vector<vector<u8>>): vector<u8> {
        let result = vector::empty<u8>();
        let i = 0;
        while (i < vector::length(ethAddrs)) {
            vector::append(&mut result, hexToString(vector::borrow(ethAddrs, i), true));
            vector::append(&mut result, b"\n");
            i = i + 1;
        };
        result
    }

    fun addressToU256(addr: vector<u8>): u256 {
        let value = 0;
        let i = 0;
        while (i < vector::length(&addr)) {
            value = value << 8;
            value = value + (*vector::borrow(&addr, i) as u256);
            i = i + 1;
        };
        value
    }

    fun cmpAddrList(list1: vector<vector<u8>>, list2: vector<vector<u8>>): bool {
        if (vector::length(&list1) > vector::length(&list2)) {
            true
        } else if (vector::length(&list1) < vector::length(&list2)) {
            false
        } else {
            let i = 0;
            while (i < vector::length(&list1)) {
                let addr1U256 = addressToU256(*vector::borrow(&list1, i));
                let addr2U256 = addressToU256(*vector::borrow(&list2, i));
                if (addr1U256 > addr2U256) {
                    return true
                } else if (addr1U256 < addr2U256) {
                    return false
                };
                i = i + 1;
            };
            false
        }
    }

    public(friend) fun checkMultiSignatures(
        msg: vector<u8>,
        r: vector<vector<u8>>,
        yParityAndS: vector<vector<u8>>,
        executors: vector<vector<u8>>,
        exeIndex: u64,
    ) {
        assert!(vector::length(&r) == vector::length(&yParityAndS), EARRAY_LENGTH_NOT_EQUAL);
        assert!(vector::length(&r) == vector::length(&executors), EARRAY_LENGTH_NOT_EQUAL);
        checkExecutorsForIndex(&executors, exeIndex);
        let i = 0;
        while (i < vector::length(&executors)) {
            checkSignature(msg, *vector::borrow(&r, i), *vector::borrow(&yParityAndS, i), *vector::borrow(&executors, i));
            i = i + 1;
        };
    }

    fun checkExecutorsNotDuplicated(executors: vector<vector<u8>>) {
        let i = 0;
        while (i < vector::length(&executors)) {
            let executor = *vector::borrow(&executors, i);
            let j = 0;
            while (j < i) {
                assert!(*vector::borrow(&executors, j) != executor, EDUPLICATED_EXECUTORS);
                j = j + 1;
            };
            i = i + 1;
        };
    }

    fun checkExecutorsForIndex(executors: &vector<vector<u8>>, exeIndex: u64) {
        let storeP = account::borrow_mut_resource<PermissionsStorage>(@free_tunnel_rooch);
        assertEthAddressList(executors);
        assert!(
            vector::length(executors) >= *vector::borrow(&storeP._exeThresholdForIndex, exeIndex), 
            ENOT_MEET_THRESHOLD
        );
        let activeSince = *vector::borrow(&storeP._exeActiveSinceForIndex, exeIndex);
        assert!(activeSince < now_seconds(), EEXECUTORS_NOT_YET_ACTIVE);

        if (vector::length(&storeP._exeActiveSinceForIndex) > exeIndex + 1) {
            let nextActiveSince = *vector::borrow(&storeP._exeActiveSinceForIndex, exeIndex + 1);
            assert!(nextActiveSince > now_seconds(), EEXECUTORS_OF_NEXT_INDEX_IS_ACTIVE);
        };

        let currentExecutors = *vector::borrow(&storeP._executorsForIndex, exeIndex);
        let i = 0;
        while (i < vector::length(executors)) {
            let executor = *vector::borrow(executors, i);
            let j = 0;
            while (j < i) {
                assert!(*vector::borrow(executors, j) != executor, EDUPLICATED_EXECUTORS);
                j = j + 1;
            };
            let isExecutor = false;
            let j = 0;
            while (j < vector::length(&currentExecutors)) {
                if (executor == *vector::borrow(&currentExecutors, j)) {
                    isExecutor = true;
                    break
                };
                j = j + 1;
            };
            assert!(isExecutor, ENON_EXECUTOR);
            i = i + 1;
        };
    }

    fun checkSignature(msg: vector<u8>, r: vector<u8>, yParityAndS: vector<u8>, ethSigner: vector<u8>) {
        assert!(ethSigner != ETH_ZERO_ADDRESS, ESIGNER_CANNOT_BE_EMPTY_ADDRESS);
        assert!(vector::length(&r) == 32, EINVALID_LENGTH);
        assert!(vector::length(&yParityAndS) == 32, EINVALID_LENGTH);
        assert!(vector::length(&ethSigner) == 20, EINVALID_LENGTH);
        let recoveredEthAddr = recoverEthAddress(msg, r, yParityAndS);
        assert!(recoveredEthAddr == ethSigner, EINVALID_SIGNATURE);
    }

    #[test]
    fun testJoinAddressList() {
        let addrs = vector[
            x"00112233445566778899aabbccddeeff00112233",
            x"000000000000000000000000000000000000beef"
        ];
        let result = joinAddressList(&addrs);
        let expected =
        b"0x00112233445566778899aabbccddeeff00112233\n0x000000000000000000000000000000000000beef\n";
        assert!(result == expected, 1);
        assert!(vector::length(&expected) == 43 * 2, 1);
    }

    #[test]
    fun testAddressToU256() {
        let addr = x"00112233445566778899aabbccddeeff00112233";
        let value = addressToU256(addr);
        assert!(value == 0x00112233445566778899aabbccddeeff00112233, 1);
    }

    #[test]
    fun testVectorCompare() {
        assert!(vector[1, 2, 3] == vector[1, 2, 3], 1);
        assert!(vector[1, 2, 3] != vector[1, 2, 4], 1);
    }

    #[test]
    fun testCmpAddrList() {
        let ethAddr1 = x"00112233445566778899aabbccddeeff00112233";
        let ethAddr2 = x"00112233445566778899aabbccddeeff00112234";
        let ethAddr3 = x"0000ffffffffffffffffffffffffffffffffffff";
        assert!(cmpAddrList(vector[ethAddr1, ethAddr2], vector[ethAddr1]), 1);
        assert!(!cmpAddrList(vector[ethAddr1], vector[ethAddr1, ethAddr2]), 1);
        assert!(cmpAddrList(vector[ethAddr1, ethAddr2], vector[ethAddr1, ethAddr1]), 1);
        assert!(!cmpAddrList(vector[ethAddr2, ethAddr1], vector[ethAddr2, ethAddr2]), 1);
        assert!(!cmpAddrList(vector[ethAddr2, ethAddr3], vector[ethAddr2, ethAddr3]), 1);
    }

}