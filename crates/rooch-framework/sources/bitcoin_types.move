// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_types{
    use std::vector;
    use std::option::{Self, Option};
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use rooch_framework::bitcoin_script_buf::{Self, ScriptBuf};
    use rooch_framework::multichain_address;

    const LOCK_TIME_THRESHOLD: u32 = 500_000_000;
    const TAPROOT_ANNEX_PREFIX: u8 = 0x50;

    #[data_struct]
    struct Block has store, copy, drop {
        /// The block header
        header: Header,
        /// List of transactions contained in the block
        txdata: vector<Transaction>,
    }

    public fun header(self: &Block) : &Header {
        &self.header
    }

    public fun txdata(self: &Block) : &vector<Transaction> {
        &self.txdata
    }

    #[data_struct]
    struct Header has store, copy, drop {
        /// Block version, now repurposed for soft fork signalling.
        version: u32,
        /// Reference to the previous block in the chain.
        prev_blockhash: address,
        /// The root hash of the merkle tree of transactions in the block.
        merkle_root: address,
        /// The timestamp of the block, as claimed by the miner.
        time: u32,
        /// The target value below which the blockhash must lie.
        bits: u32,
        /// The nonce, selected to obtain a low enough blockhash.
        nonce: u32,
    }

    public fun version(self: &Header) : u32 {
        self.version
    }

    public fun prev_blockhash(self: &Header) : address {
        self.prev_blockhash
    }

    public fun merkle_root(self: &Header) : address {
        self.merkle_root
    }

    public fun time(self: &Header) : u32 {
        self.time
    }

    public fun bits(self: &Header) : u32 {
        self.bits
    }

    public fun nonce(self: &Header) : u32 {
        self.nonce
    }

    #[data_struct] 
    struct Transaction has store, copy, drop {
        /// The txid
        /// the original bitcoin::Transaction do not include txid, we add it for convenience
        id: address,
        /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
        version: u32,
        /// Block height or timestamp. Transaction cannot be included in a block until this height/time.
        ///
        /// ### Relevant BIPs
        ///
        /// * [BIP-65 OP_CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
        /// * [BIP-113 Median time-past as endpoint for lock-time calculations](https://github.com/bitcoin/bips/blob/master/bip-0113.mediawiki)
        lock_time: u32,
        /// List of transaction inputs.
        input: vector<TxIn>,
        /// List of transaction outputs.
        output: vector<TxOut>,
    }

    public fun tx_id(self: &Transaction) : address {
        self.id
    }

    public fun tx_version(self: &Transaction) : u32 {
        self.version
    }

    public fun tx_lock_time(self: &Transaction) : u32 {
        self.lock_time
    }

    public fun tx_input(self: &Transaction) : &vector<TxIn> {
        &self.input
    }

    public fun tx_output(self: &Transaction) : &vector<TxOut> {
        &self.output
    }

    #[data_struct]
    struct TxIn has store, copy, drop {
        /// The reference to the previous output that is being used as an input.
        previous_output: OutPoint,
        /// The script which pushes values on the stack which will cause
        /// the referenced output's script to be accepted.
        script_sig: vector<u8>,
        /// The sequence number, which suggests to miners which of two
        /// conflicting transactions should be preferred, or 0xFFFFFFFF
        /// to ignore this feature. This is generally never used since
        /// the miner behavior cannot be enforced.
        sequence: u32,
        /// Witness data: an array of byte-arrays.
        /// Note that this field is *not* (de)serialized with the rest of the TxIn in
        /// Encodable/Decodable, as it is (de)serialized at the end of the full
        /// Transaction. It *is* (de)serialized with the rest of the TxIn in other
        /// (de)serialization routines.
        witness: Witness,
    }

    public fun txin_previous_output(self: &TxIn) : &OutPoint {
        &self.previous_output
    }

    public fun txin_script_sig(self: &TxIn) : &vector<u8> {
        &self.script_sig
    }

    public fun txin_sequence(self: &TxIn) : u32 {
        self.sequence
    }

    public fun txin_witness(self: &TxIn) : &Witness {
        &self.witness
    }

     #[data_struct]
    struct Witness has store, copy, drop {
        witness: vector<vector<u8>>,
    }

    public fun witness_nth(self: &Witness, nth: u64) : &vector<u8> {
        vector::borrow(&self.witness, nth)
    }

    public fun witness_len(self: &Witness) : u64 {
        vector::length(&self.witness)
    }

    /// Get Tapscript following BIP341 rules regarding accounting for an annex.
    ///
    /// This does not guarantee that this represents a P2TR [`Witness`]. It
    /// merely gets the second to last or third to last element depending on
    /// the first byte of the last element being equal to 0x50. See
    /// bitcoin_script::is_v1_p2tr to check whether this is actually a Taproot witness.
    public fun witness_tapscript(self: &Witness) : Option<ScriptBuf> {
        let len = vector::length(&self.witness);
        let script_pos_from_last = 2;
        let script_buf = option::none<ScriptBuf>();
        let idx = 0;
        while(idx < len) {
            let elem = vector::borrow(&self.witness, idx);
            if (idx == len - 1 && len >= 2 && vector::length(elem)>0 && *vector::borrow(elem,0) == TAPROOT_ANNEX_PREFIX) {
                script_pos_from_last = 3;
            };
            if (len >= script_pos_from_last && idx == len - script_pos_from_last) {
                option::fill(&mut script_buf, bitcoin_script_buf::new(*elem));
            };
            idx = idx + 1;
        };
        script_buf
    }

    #[data_struct]
    struct OutPoint has store, copy, drop {
        /// The referenced transaction's txid.
        /// Use address to represent sha256d hash
        txid: address,
        /// The index of the referenced output in its transaction's vout.
        vout: u32,
    }

    public fun new_outpoint(txid: address, vout: u32) : OutPoint {
        OutPoint{txid, vout}
    }

    public fun outpoint_txid(self: &OutPoint) : address {
        self.txid
    }

    public fun outpoint_vout(self: &OutPoint) : u32 {
        self.vout
    }

    public fun unpack_outpoint(self: OutPoint) : (address, u32) {
        (self.txid, self.vout)
    }

    #[data_struct]
    struct TxOut has store, copy, drop{
        /// The value of the output, in satoshis.
        value: u64,
        /// The script which must be satisfied for the output to be spent.
        script_pubkey: ScriptBuf,
        /// The address of the output, if known. Otherwise, the Address bytes will be empty.
        /// We can not use Option<BitcoinAddress> here, because Option is not a #[data_struct]
        recipient_address: BitcoinAddress,
    }

    public fun txout_value(self: &TxOut) : u64 {
        self.value
    }

    public fun txout_script_pubkey(self: &TxOut) : &ScriptBuf {
        &self.script_pubkey
    }

    public fun txout_address(self: &TxOut) : Option<BitcoinAddress> {
        if (bitcoin_address::is_empty(&self.recipient_address)) {
            option::none()
        }else{
            option::some(self.recipient_address)
        }
    }

    public fun txout_object_address(self: &TxOut) : address {
        if (bitcoin_address::is_empty(&self.recipient_address)) {
            @rooch_framework
        }else{
            multichain_address::mapping_to_rooch_address(multichain_address::from_bitcoin(self.recipient_address))
        }
    }

    public fun unpack_txout(self: TxOut) : (u64, ScriptBuf) {
        (self.value, self.script_pubkey)
    }      

}