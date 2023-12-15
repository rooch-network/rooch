module bitcoin_move::script_buf{
    use std::vector;
    use std::option::{Self, Option};
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use bitcoin_move::opcode;

    #[data_struct]
    struct ScriptBuf has store, copy, drop {
        bytes: vector<u8>,
    }

    public fun new(bytes: vector<u8>): ScriptBuf{
        ScriptBuf{bytes: bytes}
    }

    public fun bytes(self: &ScriptBuf): &vector<u8>{
        &self.bytes
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
        sub_vector(&self.bytes, 2, 22)
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
        sub_vector(&self.bytes, 3, 23)
    }

    public fun is_witness_program(self: &ScriptBuf): bool{
        let script_len = vector::length(&self.bytes);

        let version = *vector::borrow(&self.bytes,0);
        let push_opbyte = *vector::borrow(&self.bytes,1);
        
        version <= opcode::op_pushbytes_16()
            && push_opbyte >= opcode::op_pushbytes_2()
            && push_opbyte <= opcode::op_pushbytes_40()
            && push_opbyte == ((script_len - 2) as u8)
    }

    /// Get the witness program from a witness program script.
    public fun witness_program(self: &ScriptBuf): vector<u8>{
        sub_vector(&self.bytes, 2, vector::length(&self.bytes))
    }

     /// try to get a BitcoinAddress from a ScriptBuf.
    public fun get_address(s: &ScriptBuf): Option<BitcoinAddress> {
         //TODO sync the implementation from rust.
        if(is_p2pkh(s)){
            let pubkey_hash = p2pkh_pubkey_hash(s);
            option::some(bitcoin_address::new_p2pkh(pubkey_hash))
        }else if(is_p2sh(s)){
            let script_hash = p2sh_script_hash(s);
            option::some(bitcoin_address::new_p2sh(script_hash))
        }else if(is_witness_program(s)){
            let program = witness_program(s);
            option::some(bitcoin_address::new_witness_program(program))
        }else{
            option::none()
        }
    }

    //TODO put this function in a more general module
    fun sub_vector(bytes: &vector<u8>, start: u64, end: u64): vector<u8>{
        let result = vector::empty();
        let i = start;
        while(i < end) {
            vector::push_back(&mut result, *vector::borrow(bytes, i));
            i = i + 1;
        };
        result
    }


     #[test]
    fun test_get_address_p2pkh(){
        let script_buf = script_buf::new(x"76a914010966776006953d5567439e5e39f86a0d273bee88ac");
        let addr_opt = get_address(&script_buf);
        assert!(option::is_some(&addr_opt), 1000);
        let addr = option::extract(&mut addr_opt);
        assert!(bitcoin_address::is_p2pkh(&addr), 1001);
        let addr_bytes = bitcoin_address::into_bytes(addr);
        std::debug::print(&addr_bytes);
        let expected_addr_bytes = x"00010966776006953d5567439e5e39f86a0d273bee";
        assert!(addr_bytes == expected_addr_bytes, 1002);
    }

    #[test]
    fun test_get_address_p2sh(){
        let script_buf = script_buf::new(x"a91474d691da1574e6b3c192ecfb52cc8984ee7b6c4887");
        let addr_opt = get_address(&script_buf);
        assert!(option::is_some(&addr_opt), 1000);
        let addr = option::extract(&mut addr_opt);
        assert!(is_p2sh(&addr), 1001);
        let addr_bytes = into_bytes(addr);
        std::debug::print(&addr_bytes);
        let expected_addr_bytes = x"0574d691da1574e6b3c192ecfb52cc8984ee7b6c48";
        assert!(addr_bytes == expected_addr_bytes, 1002);
    }

    #[test]
    fun test_p2wpkh_address(){
        let script_buf = script_buf::new(x"001497cdff4fd3ed6f885d54a52b79d7a2141072ae3f");
        let addr_opt = get_address(&script_buf);
        assert!(option::is_some(&addr_opt), 1000);
        let addr = option::extract(&mut addr_opt);
        assert!(bitcoin_address::is_witness_program(&addr), 1001);
        let addr_bytes = bitcoin_address::into_bytes(addr);
        //std::debug::print(&addr_bytes);
        let expected_addr_bytes = x"97cdff4fd3ed6f885d54a52b79d7a2141072ae3f";
        assert!(addr_bytes == expected_addr_bytes, 1002);
    }

    #[test]
    fun test_fail_address_get_address() {

        let bad_p2wpkh = script_buf::new(x"0014dbc5b0a8f9d4353b4b54c3db48846bb15abfec");
        let bad_p2wsh = script_buf::new(x"00202d4fa2eb233d008cc83206fa2f4f2e60199000f5b857a835e3172323385623");
        //let invalid_segwitv0_script = script_buf::new(x"001161458e330389cd0437ee9fe3641d70cc18");
        let expected = option::none<BitcoinAddress>();

        assert!(Self::get_address(&bad_p2wpkh) == expected, 1000);
        assert!(Self::get_address(&bad_p2wsh) == expected, 1001);
        //TODO fix this test
        //assert!(Self::get_address(&invalid_segwitv0_script) == expected, 1002);
    }
}