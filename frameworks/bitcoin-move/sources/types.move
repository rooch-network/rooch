// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::types{
    use std::vector;
    use std::option::{Self, Option};
    use moveos_std::address;
    use moveos_std::bcs;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use bitcoin_move::script_buf::{Self, ScriptBuf};
    use bitcoin_move::bitcoin_hash;

    const LOCK_TIME_THRESHOLD: u32 = 500_000_000;
    const TAPROOT_ANNEX_PREFIX: u8 = 0x50;
    const U32_MAX: u32 = 4_294_967_295u32;

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

    public fun unpack_block(self: Block) : (Header, vector<Transaction>) {
        (self.header, self.txdata)
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

    public fun header_to_bytes(self: &Header) : vector<u8> {
        let output = vector::empty<u8>();
        vector::append(&mut output, bcs::to_bytes(&self.version));
        vector::append(&mut output, address::to_bytes(&self.prev_blockhash));
        vector::append(&mut output, address::to_bytes(&self.merkle_root));
        vector::append(&mut output, bcs::to_bytes(&self.time));
        vector::append(&mut output, bcs::to_bytes(&self.bits));
        vector::append(&mut output, bcs::to_bytes(&self.nonce));
        output
    }

    public fun header_to_hash(self: &Header) : address {
        let header = header_to_bytes(self);
        bitcoin_hash::sha256d(header)
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
                option::fill(&mut script_buf, script_buf::new(*elem));
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

    /// Creates a "null" `OutPoint`.
    /// This value is used for coinbase transactions because they don't have any previous outputs.
    public fun null_outpoint(): OutPoint {
        OutPoint{
            txid: address::zero(),
            vout: U32_MAX,
        }
    }

    public fun is_null_outpoint(self: &OutPoint) : bool {
        *self == null_outpoint()
    }

    #[data_struct]
    struct TxOut has store, copy, drop{
        /// The value of the output, in satoshis.
        value: u64,
        /// The script which must be satisfied for the output to be spent.
        script_pubkey: ScriptBuf,
        /// The address of the output, if known. Otherwise, the Address bytes will be empty.
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
            @bitcoin_move
        }else{
            bitcoin_address::to_rooch_address(&self.recipient_address)
        }
    }

    public fun unpack_txout(self: TxOut) : (u64, ScriptBuf) {
        (self.value, self.script_pubkey)
    }

    #[data_struct]
    struct BlockHeightHash has copy, store, drop{
        block_height: u64,
        block_hash: address,
    }

    public fun new_block_height_hash(block_height: u64, block_hash: address) : BlockHeightHash {
        BlockHeightHash{block_height, block_hash}
    }

    public fun unpack_block_height_hash(self: BlockHeightHash) : (u64, address) {
        (self.block_height, self.block_hash)
    }

    public fun is_coinbase_tx(tx: &Transaction): bool {
        let is_coinbase = if(vector::length(&tx.input) == 1) {
            let first_input = vector::borrow(&tx.input, 0);
            is_null_outpoint(&first_input.previous_output)
        } else {
            false
        };
        is_coinbase
    }      

    #[test_only]
    public fun new_header_for_test(version: u32, prev_blockhash: address, merkle_root: address, time: u32, bits: u32, nonce: u32) : Header {
        Header{
            version,
            prev_blockhash,
            merkle_root,
            time,
            bits,
            nonce,
        }
    }

    #[test_only]
    public fun new_coinbase_tx_for_test(miner: BitcoinAddress) : Transaction {
        //TODO calculate txid via Bitcoin transaction data
        let id = moveos_std::tx_context::fresh_address_for_testing();
        let input = vector::singleton(TxIn{
            previous_output: null_outpoint(),
            script_sig: vector::empty(),
            sequence: U32_MAX,
            witness: Witness{witness: vector::empty()},
        });
        let output = vector::singleton(TxOut{
            value: 0,
            //TODO construct script_pubkey
            script_pubkey: script_buf::new(vector::empty()),
            recipient_address: miner,
        });
        Transaction{
            id,
            version: 2u32,
            lock_time: 0u32,
            input,
            output,
        }
    }

    #[test_only]
    public fun new_block_for_test(header: Header, txdata: vector<Transaction>) : Block {
        Block{
            header,
            txdata,
        }
    }

    #[test_only]
    public fun fake_block_for_test(time: u32, miner: BitcoinAddress): Block{
        let prev_blockhash = moveos_std::tx_context::fresh_address_for_testing();
        let merkle_root = moveos_std::tx_context::fresh_address_for_testing();
        let bits = 0x1d00ffff;
        let nonce = 0x00000000;
        let header = new_header_for_test(0x2000_0000, prev_blockhash, merkle_root, time, bits, nonce);
        let coinbase_tx = new_coinbase_tx_for_test(miner);
        let txdata = vector::singleton(coinbase_tx);
        new_block_for_test(header, txdata) 
    }

    #[test]
    fun test_block_decode(){
        //https://mempool.space/block/00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7
        let block_bytes = x"010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b0221da2ae8cc773b020b4873f597369416cf961a1896c24106b0198459fec2df770100000000000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0804ffff001d026e04ffffffff000100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bdef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99c48b0e3f39c51347a19a5819651503a5ac00339d9a371e2b5a26147ddfd87228b900ff75762a18a40f2778bedbcde7e9b0a301000000000000000321f75f3139a013f50f315b23b0c9a2b6eac31e2bec98e5891c924664889942260000000049483045022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067fa6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a5014081b5c01ffffffff0079cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b84d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea5982f88cef651e9354f5791602369bf5a82a6cd61a62501ffffffff00fe09f5fe3ffbf5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e82000000004847304402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d33812a5ce01ffffffff0001009d966b01000000434104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac00";
        let block: Block = bcs::from_bytes(block_bytes);
        //std::debug::print(&block.header.prev_blockhash);
        let expected_prev_hash = bitcoin_hash::from_ascii_bytes(&b"00000000e47349de5a0193abc5a2fe0be81cb1d1987e45ab85f3289d54cddc4d");
        //std::debug::print(&expected_prev_hash);
        assert!(block.header.prev_blockhash == expected_prev_hash, 1);
        let expected_hash = bitcoin_hash::from_ascii_bytes(&b"00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7");
        assert!(header_to_hash(&block.header) == expected_hash, 2);
        assert!(is_coinbase_tx(vector::borrow(&block.txdata, 0)), 3);
    }

    #[test]
    fun test_block_header_hash() {
        //https://mempool.space/block/00000000000000000002b73f69e81b8b5e98dff0f2b7632fcb83c050c3b099a1
        let version = 536879108;
        let prev_blockhash =  bitcoin_hash::from_ascii_bytes(&b"00000000000000000009d54a110cc122960d31567d3ee84a1f18a98f50591046");
        let merkle_root = bitcoin_hash::from_ascii_bytes(&b"e1e0573e6098d8128ee859e7540f56b01fe0a33e56694df6d2fab0f96c4954b3");

        let time = 1644403033;
        let bits = 0x170a8bb4;
        let nonce =  1693537958;

        let block_header = new_header_for_test(version, prev_blockhash, merkle_root, time, bits, nonce);
        let block_hash = header_to_hash(&block_header);
        std::debug::print(&block_hash);
        assert!(block_hash == bitcoin_hash::from_ascii_bytes(&b"00000000000000000002b73f69e81b8b5e98dff0f2b7632fcb83c050c3b099a1"), 1);
    }
}