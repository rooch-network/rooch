// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::script_buf{
    use std::vector;
    use bitcoin_move::opcode;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};

    const ErrorInvalidKeySize: u64 = 1;
    const ErrorNumberOverflow: u64 = 2;
    const ErrorInvalidPubkeyHash: u64 = 3;
    const ErrorInvalidScriptHash: u64 = 4;

    const BITCOIN_X_ONLY_PUBKEY_SIZE: u64 = 32;
    const BITCOIN_PUBKEY_SIZE: u64 = 33;
    const BITCOIN_PUBKEY_HASH_SIZE: u64 = 20;
    const BITCOIN_SCRIPT_HASH_SIZE: u64 = 20;

    const I64_MAX: u64 = 9223372036854775807;

    #[data_struct]
    struct ScriptBuf has store, copy, drop {
        bytes: vector<u8>,
    }

    public fun empty(): ScriptBuf{
        ScriptBuf{bytes: vector::empty()}
    }

    public fun new(bytes: vector<u8>): ScriptBuf{
        ScriptBuf{bytes: bytes}
    }

    public fun single(opcode: u8): ScriptBuf{
        ScriptBuf{bytes: vector::singleton(opcode)}
    }
   
    public fun new_p2pkh(pubkey_hash: vector<u8>): ScriptBuf{
        assert!(vector::length(&pubkey_hash) == BITCOIN_PUBKEY_HASH_SIZE, ErrorInvalidPubkeyHash);
        let sb = empty();
        push_opcode(&mut sb, opcode::op_dup());
        push_opcode(&mut sb, opcode::op_hash160());
        push_data(&mut sb, pubkey_hash);
        push_opcode(&mut sb, opcode::op_equalverify());
        push_opcode(&mut sb, opcode::op_checksig());
        sb
    }


    public fun new_p2sh(script_hash: vector<u8>): ScriptBuf{
        assert!(vector::length(&script_hash) == BITCOIN_SCRIPT_HASH_SIZE, ErrorInvalidScriptHash);
        let sb = empty();
        push_opcode(&mut sb, opcode::op_hash160());
        push_data(&mut sb, script_hash);
        push_opcode(&mut sb, opcode::op_equal());
        sb
    }

    fun new_witness_program_unchecked(version: u8, program: vector<u8>): ScriptBuf{
        let sb = empty();
        if(version == 0){
            push_opcode(&mut sb, opcode::op_pushbytes_0());
        } else {
            push_opcode(&mut sb, opcode::op_pushnum_1() + version - 1);
        };
        push_data(&mut sb, program);
        sb
    }

    /// Generates a script pubkey spending to this address.
    public fun script_pubkey(addr: &BitcoinAddress): ScriptBuf {
        let address_type = bitcoin_address::pay_load_type(addr);
        let payload = bitcoin_address::pay_load(addr);
        if (address_type == bitcoin_address::pay_load_type_pubkey_hash()) {
            new_p2pkh(payload)
        } else if (address_type == bitcoin_address::pay_load_type_script_hash()) {
            new_p2sh(payload)
        } else {
            let version = *vector::borrow(&payload, 0);
            let program = vector::slice(&payload, 1, vector::length(&payload));
            new_witness_program_unchecked(version, program)
        }
    }

    /// Returns true if the address creates a particular script
    public fun match_script_pubkey(addr: &BitcoinAddress, sb: &ScriptBuf): bool {
        let address_type = bitcoin_address::pay_load_type(addr);
        let payload = bitcoin_address::pay_load(addr);
        if (address_type == bitcoin_address::pay_load_type_pubkey_hash()) {
            is_p2pkh(sb) && p2pkh_pubkey_hash(sb) == payload
        } else if (address_type == bitcoin_address::pay_load_type_script_hash()) {
            is_p2sh(sb) && p2sh_script_hash(sb) == payload
        } else {
            let program = vector::slice(&payload, 1, vector::length(&payload));
            is_witness_program(sb) && witness_program(sb) == program
        }
    }

    public fun is_empty(self: &ScriptBuf): bool {
        vector::is_empty(&self.bytes)
    }

    public fun bytes(self: &ScriptBuf): &vector<u8>{
        &self.bytes
    }

    public fun into_bytes(self: ScriptBuf): vector<u8>{
        self.bytes
    }

    /// Checks if the given script is a P2SH script.
    public fun is_p2sh(self: &ScriptBuf): bool{
        vector::length(&self.bytes) == 23 &&
            *vector::borrow(&self.bytes,0) == opcode::op_hash160() &&
            *vector::borrow(&self.bytes,1) == opcode::op_pushbytes_20() &&
            *vector::borrow(&self.bytes,22) == opcode::op_equal()
    }

    /// Get the script hash from a P2SH script.
    /// This function does not check if the script is a P2SH script, the caller must do that.
    public fun p2sh_script_hash(self: &ScriptBuf): vector<u8>{
        vector::slice(&self.bytes, 2, 22)
    }

    /// Checks if the given script is a P2PKH script.
    public fun is_p2pkh(self: &ScriptBuf): bool{
        vector::length(&self.bytes) == 25 &&
            *vector::borrow(&self.bytes,0) == opcode::op_dup() &&
            *vector::borrow(&self.bytes,1) == opcode::op_hash160() &&
            *vector::borrow(&self.bytes,2) == opcode::op_pushbytes_20() &&
            *vector::borrow(&self.bytes,23) == opcode::op_equalverify() &&
            *vector::borrow(&self.bytes,24) == opcode::op_checksig()
    }

    /// Get the public key hash from a P2PKH script.
    /// This function does not check if the script is a P2PKH script, the caller must do that.
    public fun p2pkh_pubkey_hash(self: &ScriptBuf): vector<u8>{
        vector::slice(&self.bytes, 3, 23)
    }

    public fun is_witness_program(self: &ScriptBuf): bool{
        let script_len = vector::length(&self.bytes);

        let version = *vector::borrow(&self.bytes,0);
        
        if (!(version == opcode::op_pushbytes_0() || (version >= opcode::op_pushnum_1() && version <= opcode::op_pushnum_16()))){
            return false
        };

        let push_opbyte = *vector::borrow(&self.bytes,1);
        push_opbyte >= opcode::op_pushbytes_2()
            && push_opbyte <= opcode::op_pushbytes_40()
            && push_opbyte == ((script_len - 2) as u8)
    }

    /// Get the witness program from a witness program script.
    public fun witness_program(self: &ScriptBuf): vector<u8>{
        vector::slice(&self.bytes, 2, vector::length(&self.bytes))
    }


    /// Checks if the given script is an OP_RETURN script.
    public fun is_op_return(self: &ScriptBuf): bool {
        vector::length(&self.bytes) > 0 &&
            *vector::borrow(&self.bytes, 0) == opcode::op_return()
    }

    // ====== Script Builder ======

    public fun push_opcode(self: &mut ScriptBuf, opcode: u8) {
        vector::push_back(&mut self.bytes,opcode);
    }

    public fun push_data(self: &mut ScriptBuf, data: vector<u8>) {
        let len = vector::length(&data);
        if (len < 76) {
            //OP_PUSHBYTES_x
            vector::push_back(&mut self.bytes, (len as u8));
        } else if (len < 0x100) {
            vector::push_back(&mut self.bytes, opcode::op_pushdata1());
            vector::push_back(&mut self.bytes, (len as u8));
        } else if (len < 0x10000) {
            vector::push_back(&mut self.bytes, opcode::op_pushdata2());
            vector::push_back(&mut self.bytes, ((len & 0xff) as u8));
            vector::push_back(&mut self.bytes, ((len >> 8) as u8));
        } else {
            vector::push_back(&mut self.bytes, opcode::op_pushdata4());
            vector::push_back(&mut self.bytes, ((len & 0xff) as u8));
            vector::push_back(&mut self.bytes, (((len >> 8) & 0xff) as u8));
            vector::push_back(&mut self.bytes, ((len >> 16) as u8));
        };
        vector::append(&mut self.bytes, data);
    }

    /// Adds instructions to push an integer onto the stack.
    ///
    /// Integers are encoded as little-endian signed-magnitude numbers, but there are dedicated
    /// opcodes to push some small integers.
    /// Because there no i64 type in Move, we use u64 to represent the integer.
    /// The value over the I64_MAX will abort, we can support negative value in the future.
    public fun push_int(self: &mut ScriptBuf, n: u64) {
        if (n > I64_MAX) {
            abort ErrorNumberOverflow
        };
        push_signed_int(self, n, false);
    }

    //TODO: design a better api to support negative value.
    fun push_signed_int(self: &mut ScriptBuf, abs: u64, neg: bool) {
        if (neg && abs == 1){
            vector::push_back(&mut self.bytes, opcode::op_pushnum_neg1());
        } else if (abs == 0) {
            vector::push_back(&mut self.bytes, opcode::op_0());
        } else if (!neg && abs <= 16) {
            let n = (abs as u8);
            vector::push_back(&mut self.bytes, opcode::op_pushnum_1() + n - 1);
        } else {
            push_int_non_minimal(self, abs, neg);
        } 
    }

    /// Adds instructions to push an integer onto the stack without optimization.
    ///
    /// This uses the explicit encoding regardless of the availability of dedicated opcodes.
    fun push_int_non_minimal(self: &mut ScriptBuf, abs: u64, neg: bool) {
        let data = encode_script_int(abs, neg);
        push_data(self, data)
    }

    /// Encodes an integer in script(minimal CScriptNum) format.
    /// [`CScriptNum::serialize`]: <https://github.com/bitcoin/bitcoin/blob/8ae2808a4354e8dcc697f76bacc5e2f2befe9220/src/script/script.h#L345>
    fun encode_script_int(abs: u64, neg: bool): vector<u8> {
        let data = vector::empty<u8>();
        if (abs == 0) {
            return data
        };

        while (abs > 0) {
            vector::push_back(&mut data, ((abs & 0xFF) as u8));
            abs = abs >> 8;
        };

        let len = vector::length(&data);
        let last_byte = *vector::borrow(&data, len - 1);
        if ((last_byte & 0x80) != 0) {
            vector::push_back(&mut data, if (neg) 0x80u8 else 0u8);
        } else {
            vector::pop_back(&mut data);
            vector::push_back(&mut data, last_byte | if (neg) 0x80u8 else 0u8);
        };

        data
    }

    /// Push a Bitcoin public key to the script
    public fun push_key(self: &mut ScriptBuf, key: vector<u8>) {
        assert!(vector::length(&key) == BITCOIN_PUBKEY_SIZE, ErrorInvalidKeySize);
        push_data(self, key);
    }

    /// Push a Bitcoin x-only public key to the script
    public fun push_x_only_key(self: &mut ScriptBuf, key: vector<u8>) {
        assert!(vector::length(&key) == BITCOIN_X_ONLY_PUBKEY_SIZE, ErrorInvalidKeySize);
        push_data(self, key);
    }
 
    #[test]
    fun test_push_int() {
        let inputs = vector[
            0u64, 1u64, 16u64, 17u64, 75u64, 76u64, 100u64, 255u64, 256u64, 65535u64, 65536u64, 
            16777215u64, 16777216u64, 2147483647u64, 2147483648u64, 4294967295u64, 
            4294967296u64, 1000000000000u64,
        ];
        let expected_outputs = vector[
            x"00",
            x"51",
            x"60",
            x"0111",
            x"014b",
            x"014c",
            x"0164",
            x"02ff00",
            x"020001",
            x"03ffff00",
            x"03000001",
            x"04ffffff00",
            x"0400000001",
            x"04ffffff7f",
            x"050000008000",
            x"05ffffffff00",
            x"050000000001",
            x"060010a5d4e800",
        ];

        let i = 0;
        while (i < vector::length(&inputs)) {
            let input = *vector::borrow(&inputs, i);
            let expected = vector::borrow(&expected_outputs, i);

            let sb = empty();
            push_int(&mut sb, input);
            let result = into_bytes(sb);
            assert!(result == *expected, i);
            i = i + 1;
        };
    }

    #[test]
    fun test_push_int_neg() {
        let inputs = vector[
            1u64, 100u64, 1000000000000u64,
        ];
        let expected_outputs = vector[
            x"4f",
            x"01e4",
            x"060010a5d4e880"
        ];

        let i = 0;
        while (i < vector::length(&inputs)) {
            let input = *vector::borrow(&inputs, i);
            let expected = vector::borrow(&expected_outputs, i);

            let sb = empty();
            push_signed_int(&mut sb, input, true);
            let result = into_bytes(sb);
            assert!(result == *expected, i);
            i = i + 1;
        };
    }

    #[test]
    fun test_p2pkh() {
        let addr = bitcoin_address::from_string(&std::string::utf8(b"1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY"));
        assert!(bitcoin_address::is_p2pkh(&addr), 1001);
        let sb = script_pubkey(&addr);
        assert!(is_p2pkh(&sb), 1002);
    }

    #[test]
    fun test_p2pkh_with_new() {
        let sb = new_p2pkh(x"1234567890123456789012345678901234567890");
        assert!(is_p2pkh(&sb), 1001);
        assert!(p2pkh_pubkey_hash(&sb) == x"1234567890123456789012345678901234567890", 1002);
    }

    #[test]
    fun test_p2sh(){
        let addr = bitcoin_address::from_string(&std::string::utf8(b"3QBRmWNqqBGme9er7fMkGqtZtp4gjMFxhE"));
        assert!(bitcoin_address::is_p2sh(&addr), 1001);
        let sb = script_pubkey(&addr);
        assert!(is_p2sh(&sb), 1002);
        //std::debug::print(&p2sh_script_hash(&sb));
        assert!(p2sh_script_hash(&sb) == x"f6b2517ca82f1b0ed43830d075069d6aa0b695ca", 1003);
    }

    #[test]
    fun test_witness_program(){
        let sb = new_witness_program_unchecked(0, x"1234567890123456789012345678901234567890");
        assert!(is_witness_program(&sb), 1001);
        assert!(witness_program(&sb) == x"1234567890123456789012345678901234567890", 1002);
    }

    #[test]
    fun test_witness_version_1(){
        let addr = bitcoin_address::from_string(&std::string::utf8(b"bc1zw508d6qejxtdg4y5r3zarvaryvaxxpcs"));
        assert!(bitcoin_address::is_witness_program(&addr), 1001);
        let sb = script_pubkey(&addr);
        //std::debug::print(&witness_program(&sb));
        assert!(is_witness_program(&sb), 1002);
        assert!(witness_program(&sb) == x"751e76e8199196d454941c45d1b3a323", 1003);
    }

    #[test]
    fun test_p2tr(){
        let addr = bitcoin_address::from_string(&std::string::utf8(b"bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr"));
        assert!(bitcoin_address::is_witness_program(&addr), 1001);
        let sb = script_pubkey(&addr);
        assert!(is_witness_program(&sb), 1002);
        //std::debug::print(&witness_program(&sb));
        assert!(witness_program(&sb) == x"a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6880949dc684c", 1003);
    }

    #[test]
    fun test_match_script_pubkey() {
        let addresses = vector[
            std::string::utf8(b"1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY"),
            std::string::utf8(b"1J4LVanjHMu3JkXbVrahNuQCTGCRRgfWWx"),
            std::string::utf8(b"33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k"),
            std::string::utf8(b"3QBRmWNqqBGme9er7fMkGqtZtp4gjMFxhE"),
            std::string::utf8(b"bc1zw508d6qejxtdg4y5r3zarvaryvaxxpcs"),
            std::string::utf8(b"bc1qvzvkjn4q3nszqxrv3nraga2r822xjty3ykvkuw"),
            std::string::utf8(b"bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr"),
            std::string::utf8(b"bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e")
        ];

        let i = 0;
        while (i < vector::length(&addresses)) {
            let addr_str = vector::borrow(&addresses, i);
            let addr = bitcoin_address::from_string(addr_str);
             
            let j = 0;
            while (j < vector::length(&addresses)) {
                let another_addr_str = vector::borrow(&addresses, j);
                let another_addr = bitcoin_address::from_string(another_addr_str);
                let another_script_pubkey = script_pubkey(&another_addr);
                
                assert!(
                    match_script_pubkey(&addr, &another_script_pubkey) == (i == j),
                    i * 8 + j
                );
                
                j = j + 1;
            };
            
            i = i + 1;
        };
    }
}
