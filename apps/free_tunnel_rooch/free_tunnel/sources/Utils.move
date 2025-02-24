module free_tunnel_rooch::utils {

    use std::vector;
    use moveos_std::hash;
    use rooch_framework::ecdsa_k1;

    const ETOSTRING_VALUE_TOO_LARGE: u64 = 100;
    const ELOG10_VALUE_TOO_LARGE: u64 = 101;
    const EINVALID_PUBLIC_KEY: u64 = 102;
    const EINVALID_ETH_ADDRESS: u64 = 103;

    const HEX_TO_STRING_DICT: vector<vector<u8>> = vector[
        b"00", b"01", b"02", b"03", b"04", b"05", b"06", b"07", b"08", b"09", b"0a", b"0b", b"0c", b"0d", b"0e", b"0f",
        b"10", b"11", b"12", b"13", b"14", b"15", b"16", b"17", b"18", b"19", b"1a", b"1b", b"1c", b"1d", b"1e", b"1f",
        b"20", b"21", b"22", b"23", b"24", b"25", b"26", b"27", b"28", b"29", b"2a", b"2b", b"2c", b"2d", b"2e", b"2f",
        b"30", b"31", b"32", b"33", b"34", b"35", b"36", b"37", b"38", b"39", b"3a", b"3b", b"3c", b"3d", b"3e", b"3f",
        b"40", b"41", b"42", b"43", b"44", b"45", b"46", b"47", b"48", b"49", b"4a", b"4b", b"4c", b"4d", b"4e", b"4f",
        b"50", b"51", b"52", b"53", b"54", b"55", b"56", b"57", b"58", b"59", b"5a", b"5b", b"5c", b"5d", b"5e", b"5f",
        b"60", b"61", b"62", b"63", b"64", b"65", b"66", b"67", b"68", b"69", b"6a", b"6b", b"6c", b"6d", b"6e", b"6f",
        b"70", b"71", b"72", b"73", b"74", b"75", b"76", b"77", b"78", b"79", b"7a", b"7b", b"7c", b"7d", b"7e", b"7f",
        b"80", b"81", b"82", b"83", b"84", b"85", b"86", b"87", b"88", b"89", b"8a", b"8b", b"8c", b"8d", b"8e", b"8f",
        b"90", b"91", b"92", b"93", b"94", b"95", b"96", b"97", b"98", b"99", b"9a", b"9b", b"9c", b"9d", b"9e", b"9f",
        b"a0", b"a1", b"a2", b"a3", b"a4", b"a5", b"a6", b"a7", b"a8", b"a9", b"aa", b"ab", b"ac", b"ad", b"ae", b"af",
        b"b0", b"b1", b"b2", b"b3", b"b4", b"b5", b"b6", b"b7", b"b8", b"b9", b"ba", b"bb", b"bc", b"bd", b"be", b"bf",
        b"c0", b"c1", b"c2", b"c3", b"c4", b"c5", b"c6", b"c7", b"c8", b"c9", b"ca", b"cb", b"cc", b"cd", b"ce", b"cf",
        b"d0", b"d1", b"d2", b"d3", b"d4", b"d5", b"d6", b"d7", b"d8", b"d9", b"da", b"db", b"dc", b"dd", b"de", b"df",
        b"e0", b"e1", b"e2", b"e3", b"e4", b"e5", b"e6", b"e7", b"e8", b"e9", b"ea", b"eb", b"ec", b"ed", b"ee", b"ef",
        b"f0", b"f1", b"f2", b"f3", b"f4", b"f5", b"f6", b"f7", b"f8", b"f9", b"fa", b"fb", b"fc", b"fd", b"fe", b"ff",
    ];


    public fun smallU64ToString(value: u64): vector<u8> {
        let buffer = vector::empty<u8>();
        assert!(value < 10000000000, ETOSTRING_VALUE_TOO_LARGE);
        if (value >= 1000000000) {
            let byte = ((value / 1000000000) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 100000000) {
            let byte = (((value / 100000000) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 10000000) {
            let byte = (((value / 10000000) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 1000000) {
            let byte = (((value / 1000000) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 100000) {
            let byte = (((value / 100000) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 10000) {
            let byte = (((value / 10000) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 1000) {
            let byte = (((value / 1000) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 100) {
            let byte = (((value / 100) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        if (value >= 10) {
            let byte = (((value / 10) % 10) as u8) + 48;
            vector::push_back(&mut buffer, byte);
        };
        let byte = ((value % 10) as u8) + 48;
        vector::push_back(&mut buffer, byte);
        buffer
    }

    public fun smallU64Log10(value: u64): u64 {
        assert!(value < 10000, ELOG10_VALUE_TOO_LARGE);
        let result = 0;
        value = value / 10;
        while (value != 0) {
            value = value / 10;
            result = result + 1;
        };
        result  // Returns 0 if given 0. Same as Solidity
    }

    public fun hexToString(hex: &vector<u8>, prefix: bool): vector<u8> {
        let str = vector::empty<u8>();
        if (prefix) {
            vector::append(&mut str, b"0x");
        };
        let i = 0;
        while (i < vector::length(hex)) {
            let byte = *vector::borrow(hex, i);
            vector::append(&mut str, *vector::borrow(&HEX_TO_STRING_DICT, (byte as u64)));
            i = i + 1;
        };
        str
    }

    public fun ethAddressFromPubkey(pk: vector<u8>): vector<u8> {
        assert!(vector::length(&pk) == 64, EINVALID_PUBLIC_KEY);
        let hash = hash::keccak256(&pk);
        let ethAddr = vector::empty<u8>();
        let i = 12;
        while (i < 32) {
            vector::push_back(&mut ethAddr, *vector::borrow(&hash, i));
            i = i + 1;
        };
        ethAddr
    }

    public fun recoverEthAddress(msg: vector<u8>, r: vector<u8>, yParityAndS: vector<u8>): vector<u8> {
        let s = copy yParityAndS;
        let v = *vector::borrow_mut(&mut s, 0) >> 7;
        *vector::borrow_mut(&mut s, 0) = *vector::borrow(&s, 0) & 0x7f;
        vector::append(&mut r, s);
        vector::push_back(&mut r, v);
        let compressed_pk = ecdsa_k1::ecrecover(&r, &msg, 0);
        let pk = ecdsa_k1::decompress_pubkey(&compressed_pk);
        vector::remove(&mut pk, 0); // drop '04' prefix
        ethAddressFromPubkey(pk)
    }

    fun assertEthAddress(addr: &vector<u8>) {
        assert!(vector::length(addr) == 20, EINVALID_ETH_ADDRESS);
    }

    public fun assertEthAddressList(addrs: &vector<vector<u8>>) {
        let i = 0;
        while (i < vector::length(addrs)) {
            assertEthAddress(vector::borrow(addrs, i));
            i = i + 1;
        };
    }

    #[test]
    fun testSmallU64ToString() {
        assert!(smallU64ToString(0) == b"0", 1);
        assert!(smallU64ToString(1) == b"1", 1);
        assert!(smallU64ToString(9) == b"9", 1);
        assert!(smallU64ToString(10) == b"10", 1);
        assert!(smallU64ToString(11) == b"11", 1);
        assert!(smallU64ToString(60) == b"60", 1);
        assert!(smallU64ToString(99) == b"99", 1);
        assert!(smallU64ToString(100) == b"100", 1);
        assert!(smallU64ToString(104) == b"104", 1);
        assert!(smallU64ToString(110) == b"110", 1);
        assert!(smallU64ToString(500) == b"500", 1);
        assert!(smallU64ToString(919) == b"919", 1);
        assert!(smallU64ToString(999) == b"999", 1);
        assert!(smallU64ToString(1000) == b"1000", 1);
        assert!(smallU64ToString(1001) == b"1001", 1);
        assert!(smallU64ToString(3417) == b"3417", 1);
        assert!(smallU64ToString(9283) == b"9283", 1);
        assert!(smallU64ToString(9999) == b"9999", 1);
        assert!(smallU64ToString(10000) == b"10000", 1);
        assert!(smallU64ToString(10001) == b"10001", 1);
        assert!(smallU64ToString(99999) == b"99999", 1);
        assert!(smallU64ToString(100000) == b"100000", 1);
        assert!(smallU64ToString(100001) == b"100001", 1);
        assert!(smallU64ToString(999999) == b"999999", 1);
        assert!(smallU64ToString(1000000) == b"1000000", 1);
        assert!(smallU64ToString(9999999) == b"9999999", 1);
        assert!(smallU64ToString(10000000) == b"10000000", 1);
        assert!(smallU64ToString(99999999) == b"99999999", 1);
        assert!(smallU64ToString(100000000) == b"100000000", 1);
        assert!(smallU64ToString(999999999) == b"999999999", 1);
        assert!(smallU64ToString(1000000000) == b"1000000000", 1);
        assert!(smallU64ToString(1732709334) == b"1732709334", 1);     // Timestamp of `2024-11-27 12:08:54`
        assert!(smallU64ToString(9999999999) == b"9999999999", 1);
    }

    #[test]
    #[expected_failure(abort_code = ETOSTRING_VALUE_TOO_LARGE)]
    fun testSmallU64ToStringTooLargeFailure1() {
        smallU64ToString(10000000000);
    }

    #[test]
    #[expected_failure(abort_code = ETOSTRING_VALUE_TOO_LARGE)]
    fun testSmallU64ToStringTooLargeFailure2() {
        smallU64ToString(12000000000);
    }

    #[test]
    fun testSmallU64Log10() {
        assert!(smallU64Log10(0) == 0, 1);
        assert!(smallU64Log10(1) == 0, 1);
        assert!(smallU64Log10(3) == 0, 1);
        assert!(smallU64Log10(9) == 0, 1);
        assert!(smallU64Log10(10) == 1, 1);
        assert!(smallU64Log10(35) == 1, 1);
        assert!(smallU64Log10(100) == 2, 1);
        assert!(smallU64Log10(1000) == 3, 1);
        assert!(smallU64Log10(3162) == 3, 1);
        assert!(smallU64Log10(9999) == 3, 1);
    }

    #[test]
    #[expected_failure(abort_code = ELOG10_VALUE_TOO_LARGE)]
    fun testSmallU64Log10TooLargeFailure1() {
        smallU64Log10(10000);
    }

    #[test]
    #[expected_failure(abort_code = ELOG10_VALUE_TOO_LARGE)]
    fun testSmallU64Log10TooLargeFailure2() {
        smallU64Log10(12000);
    }

    #[test]
    fun testHexToString1() {
        let value = vector[0x33, 0x45];
        assert!(hexToString(&value, false) == b"3345", 1);
    }

    #[test]
    fun testHexToString2() {
        let hex = x"052c7707093534035fc2ed60de35e11bebb6486b";
        let str = hexToString(&hex, false);
        assert!(str == b"052c7707093534035fc2ed60de35e11bebb6486b", 1);
    }

    #[test]
    fun testHexToString3() {
        let hex = x"052c7707093534035fc2ed60de35e11bebb6486b";
        let str = hexToString(&hex, true);
        assert!(str == b"0x052c7707093534035fc2ed60de35e11bebb6486b", 1);
    }

    #[test]
    fun testEthAddressFromPubkey() {
        let pk = x"5139c6f948e38d3ffa36df836016aea08f37a940a91323f2a785d17be4353e382b488d0c543c505ec40046afbb2543ba6bb56ca4e26dc6abee13e9add6b7e189";
        let ethAddr = ethAddressFromPubkey(pk);
        assert!(ethAddr == x"052c7707093534035fc2ed60de35e11bebb6486b", 1);
    }

    #[test]
    fun testRecoverEthAddress() {
        let message = b"stupid";
        let r = x"6fd862958c41d532022e404a809e92ec699bd0739f8d782ca752b07ff978f341";
        let yParityAndS = x"f43065a96dc53a21b4eb4ce96a84a7c4103e3485b0c87d868df545fcce0f3983";
        let ethAddr = recoverEthAddress(message, r, yParityAndS);
        assert!(ethAddr == x"2eF8a51F8fF129DBb874A0efB021702F59C1b211", 1);
    }

    #[test]
    fun testAssertEthAddressList() {
        let addrs = vector[
            x"052c7707093534035fc2ed60de35e11bebb6486b",
            x"052c7707093534035fc2ed60de35e11bebb6486b",
            x"052c7707093534035fc2ed60de35e11bebb6486b",
        ];
        assertEthAddressList(&addrs);
    }

    #[test]
    #[expected_failure(abort_code = EINVALID_ETH_ADDRESS)]
    fun testAssertEthAddressListFailure() {
        let addrs = vector[
            x"052c7707093534035fc2ed60de35e11bebb648",
        ];
        assertEthAddressList(&addrs);
    }
    
}