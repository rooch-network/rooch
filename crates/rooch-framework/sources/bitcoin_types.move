// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_types{

    const LOCK_TIME_THRESHOLD: u32 = 500_000_000;

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
        witness: vector<u8>,
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

    public fun txin_witness(self: &TxIn) : &vector<u8> {
        &self.witness
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
        script_pubkey: vector<u8>,
    }

    public fun txout_value(self: &TxOut) : u64 {
        self.value
    }

    public fun txout_script_pubkey(self: &TxOut) : &vector<u8> {
        &self.script_pubkey
    }

    public fun unpack_txout(self: TxOut) : (u64, vector<u8>) {
        (self.value, self.script_pubkey)
    }      
}