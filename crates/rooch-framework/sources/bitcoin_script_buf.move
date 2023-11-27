module rooch_framework::bitcoin_script_buf{
    use std::vector;
    use rooch_framework::bitcoin_opcode;

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
            *vector::borrow(&self.bytes,0) == bitcoin_opcode::op_hash160() &&
            *vector::borrow(&self.bytes,1) == bitcoin_opcode::op_pushbytes_20() &&
            *vector::borrow(&self.bytes,22) == bitcoin_opcode::op_equal()
    }

    /// Get the script hash from a P2SH script.
    /// This function does not check if the script is a P2SH script, the caller must do that.
    public fun p2sh_script_hash(self: &ScriptBuf): vector<u8>{
        sub_vector(&self.bytes, 2, 22)
    }

    /// Checks if the given script is a P2PKH script.
    public fun is_p2pkh(self: &ScriptBuf): bool{
        vector::length(&self.bytes) == 25 &&
            *vector::borrow(&self.bytes,0) == bitcoin_opcode::op_dup() &&
            *vector::borrow(&self.bytes,1) == bitcoin_opcode::op_hash160() &&
            *vector::borrow(&self.bytes,2) == bitcoin_opcode::op_pushbytes_20() &&
            *vector::borrow(&self.bytes,23) == bitcoin_opcode::op_equalverify() &&
            *vector::borrow(&self.bytes,24) == bitcoin_opcode::op_checksig()
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
        
        version <= bitcoin_opcode::op_pushbytes_16()
            && push_opbyte >= bitcoin_opcode::op_pushbytes_2()
            && push_opbyte <= bitcoin_opcode::op_pushbytes_40()
            && push_opbyte == ((script_len - 2) as u8)
    }

    /// Get the witness program from a witness program script.
    public fun witness_program(self: &ScriptBuf): vector<u8>{
        sub_vector(&self.bytes, 2, vector::length(&self.bytes))
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
}