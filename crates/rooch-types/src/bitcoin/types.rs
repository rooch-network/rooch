// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::into_address::FromAddress;
use crate::{address::BitcoinAddress, addresses::BITCOIN_MOVE_ADDRESS, into_address::IntoAddress};
use anyhow::Result;
use bitcoin::Txid;
use bitcoin::{hashes::Hash, BlockHash};
use bitcoincore_rpc::bitcoincore_rpc_json::GetBlockHeaderResult;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::h256::sha2_256_of;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("types");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// The block header
    pub header: Header,
    /// List of transactions contained in the block
    pub txdata: Vec<Transaction>,
}

impl Block {
    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode block should success")
    }
}

impl From<bitcoin::Block> for Block {
    fn from(block: bitcoin::Block) -> Self {
        Self {
            header: block.header.into(),
            txdata: block.txdata.into_iter().map(|tx| tx.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Header {
    /// Block version, now repurposed for soft fork signalling.
    pub version: u32,
    /// Reference to the previous block in the chain.
    pub prev_blockhash: AccountAddress,
    /// The root hash of the merkle tree of transactions in the block.
    pub merkle_root: AccountAddress,
    /// The timestamp of the block, as claimed by the miner.
    pub time: u32,
    /// The target value below which the blockhash must lie.
    pub bits: u32,
    /// The nonce, selected to obtain a low enough blockhash.
    pub nonce: u32,
}

impl Header {
    pub fn new(
        version: u32,
        prev_blockhash: AccountAddress,
        merkle_root: AccountAddress,
        time: u32,
        bits: u32,
        nonce: u32,
    ) -> Self {
        Self {
            version,
            prev_blockhash,
            merkle_root,
            time,
            bits,
            nonce,
        }
    }

    /// Encode the header to bytes
    /// Same as bitcoin::BlockHeader::consensus_encode
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.prev_blockhash.to_vec());
        bytes.extend_from_slice(&self.merkle_root.to_vec());
        bytes.extend_from_slice(&self.time.to_le_bytes());
        bytes.extend_from_slice(&self.bits.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes
    }

    pub fn block_hash(&self) -> AccountAddress {
        let bytes = self.encode();
        let hash = sha2_256_of(&bytes);
        let hash2 = sha2_256_of(hash.as_bytes());
        hash2.into_address()
    }
}

impl From<bitcoin::block::Header> for Header {
    fn from(header: bitcoin::block::Header) -> Self {
        Self {
            version: header.version.to_consensus() as u32,
            prev_blockhash: header.prev_blockhash.into_address(),
            merkle_root: header.merkle_root.into_address(),
            time: header.time,
            bits: header.bits.to_consensus(),
            nonce: header.nonce,
        }
    }
}

impl TryFrom<GetBlockHeaderResult> for Header {
    type Error = anyhow::Error;
    fn try_from(result: GetBlockHeaderResult) -> Result<Self> {
        let bits = i32::from_str_radix(&result.bits, 16)? as u32;
        Ok(Self {
            version: result.version.to_consensus() as u32,
            prev_blockhash: result
                .previous_block_hash
                .unwrap_or(BlockHash::all_zeros())
                .into_address(),
            merkle_root: result.merkle_root.into_address(),
            time: result.time as u32,
            bits,
            nonce: result.nonce,
        })
    }
}

impl MoveStructType for Header {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Header");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for Header {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u32::type_layout(),
            AccountAddress::type_layout(),
            AccountAddress::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// txid
    /// The original bitcoin::Transaction do not have txid field, we add it for convenience
    pub id: AccountAddress,
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    pub version: u32,
    /// Block height or timestamp. Transaction cannot be included in a block until this height/time.
    ///
    /// ### Relevant BIPs
    ///
    /// * [BIP-65 OP_CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
    /// * [BIP-113 Median time-past as endpoint for lock-time calculations](https://github.com/bitcoin/bips/blob/master/bip-0113.mediawiki)
    pub lock_time: u32,
    /// List of transaction inputs.
    pub input: Vec<TxIn>,
    /// List of transaction outputs.
    pub output: Vec<TxOut>,
}

impl From<bitcoin::Transaction> for Transaction {
    fn from(tx: bitcoin::Transaction) -> Self {
        Self {
            id: tx.txid().into_address(),
            version: tx.version.0 as u32,
            lock_time: tx.lock_time.to_consensus_u32(),
            input: tx.input.into_iter().map(|tx_in| tx_in.into()).collect(),
            output: tx.output.into_iter().map(|tx_out| tx_out.into()).collect(),
        }
    }
}

impl MoveStructType for Transaction {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Transaction");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for Transaction {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
            Vec::<TxIn>::type_layout(),
            Vec::<TxOut>::type_layout(),
        ])
    }
}

impl Transaction {
    pub fn is_coinbase(&self) -> bool {
        self.input.len() == 1 && self.input[0].previous_output.is_null()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxIn {
    /// The reference to the previous output that is being used as an input.
    pub previous_output: OutPoint,
    /// The script which pushes values on the stack which will cause
    /// the referenced output's script to be accepted.
    pub script_sig: Vec<u8>,
    /// The sequence number, which suggests to miners which of two
    /// conflicting transactions should be preferred, or 0xFFFFFFFF
    /// to ignore this feature. This is generally never used since
    /// the miner behavior cannot be enforced.
    pub sequence: u32,
    /// Witness data: an array of byte-arrays.
    /// Note that this field is *not* (de)serialized with the rest of the TxIn in
    /// Encodable/Decodable, as it is (de)serialized at the end of the full
    /// Transaction. It *is* (de)serialized with the rest of the TxIn in other
    /// (de)serialization routines.
    /// We store the decoded witness data here for convenience.
    pub witness: Witness,
}

impl From<bitcoin::TxIn> for TxIn {
    fn from(tx_in: bitcoin::TxIn) -> Self {
        Self {
            previous_output: tx_in.previous_output.into(),
            script_sig: tx_in.script_sig.into_bytes(),
            sequence: tx_in.sequence.0,
            witness: tx_in.witness.into(),
        }
    }
}

impl MoveStructType for TxIn {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TxIn");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for TxIn {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            OutPoint::type_layout(),
            Vec::<u8>::type_layout(),
            u32::type_layout(),
            Witness::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Witness {
    pub witness: Vec<Vec<u8>>,
}

impl From<bitcoin::Witness> for Witness {
    fn from(witness: bitcoin::Witness) -> Self {
        Self {
            witness: witness.to_vec(),
        }
    }
}

impl MoveStructType for Witness {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Witness");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for Witness {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![Vec::<Vec<u8>>::type_layout()])
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct OutPoint {
    /// The referenced transaction's txid.
    /// Use address to represent sha256d hash
    pub txid: AccountAddress,
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: AccountAddress, vout: u32) -> Self {
        Self { txid, vout }
    }

    pub fn null() -> Self {
        Self {
            txid: AccountAddress::ZERO,
            vout: u32::MAX,
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        *self == OutPoint::null()
    }
}

impl From<bitcoin::OutPoint> for OutPoint {
    fn from(out_point: bitcoin::OutPoint) -> Self {
        Self {
            txid: out_point.txid.into_address(),
            vout: out_point.vout,
        }
    }
}

impl From<OutPoint> for bitcoin::OutPoint {
    fn from(out_point: OutPoint) -> Self {
        Self {
            txid: Txid::from_address(out_point.txid),
            vout: out_point.vout,
        }
    }
}

impl MoveStructType for OutPoint {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("OutPoint");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for OutPoint {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
        ])
    }
}

impl Display for OutPoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        //We use bitcoin txid hex to display
        let txid = Txid::from_address(self.txid);
        write!(f, "{}:{}", txid, self.vout)
    }
}

impl FromStr for OutPoint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid OutPoint format"));
        }
        let txid = Txid::from_str(parts[0])?;
        let vout = parts[1].parse()?;
        Ok(OutPoint::new(txid.into_address(), vout))
    }
}

impl Serialize for OutPoint {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(&self)
        } else {
            #[derive(Serialize)]
            struct Value {
                txid: AccountAddress,
                vout: u32,
            }
            Value {
                txid: self.txid,
                vout: self.vout,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for OutPoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(serde::de::Error::custom)
        } else {
            #[derive(Deserialize)]
            struct Value {
                txid: AccountAddress,
                vout: u32,
            }
            let value = Value::deserialize(deserializer)?;
            Ok(OutPoint::new(value.txid, value.vout))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptBuf {
    pub bytes: Vec<u8>,
}

impl From<bitcoin::ScriptBuf> for ScriptBuf {
    fn from(script: bitcoin::ScriptBuf) -> Self {
        Self {
            bytes: script.into_bytes(),
        }
    }
}

impl MoveStructType for ScriptBuf {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ScriptBuf");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for ScriptBuf {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![Vec::<u8>::type_layout()])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxOut {
    /// The value of the output, in satoshis.
    pub value: u64,
    /// The script which must be satisfied for the output to be spent.
    pub script_pubkey: ScriptBuf,
    /// The address of the recipient
    /// We decode this from script_pubkey for convenience
    pub recipient_address: BitcoinAddress,
}

impl From<bitcoin::TxOut> for TxOut {
    fn from(tx_out: bitcoin::TxOut) -> Self {
        Self {
            value: tx_out.value.to_sat(),
            script_pubkey: tx_out.script_pubkey.clone().into(),
            recipient_address: tx_out.script_pubkey.into(),
        }
    }
}

impl MoveStructType for TxOut {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TxOut");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for TxOut {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u64::type_layout(),
            ScriptBuf::type_layout(),
            BitcoinAddress::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeightHash {
    pub block_height: u64,
    pub block_hash: AccountAddress,
}

impl MoveStructType for BlockHeightHash {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BlockHeightHash");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BlockHeightHash {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Address,
        ])
    }
}

impl BlockHeightHash {
    pub fn unpack(self) -> (u64, BlockHash) {
        let BlockHeightHash {
            block_height,
            block_hash,
        } = self;
        (block_height, BlockHash::from_address(block_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{
        block::Version,
        consensus::{deserialize, serialize},
        Amount, CompactTarget,
    };
    use hex::FromHex;
    use std::str::FromStr;

    #[test]
    fn test_header() {
        // Mainnet block https://mempool.space/block/00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7
        let block_bytes = Vec::<u8>::from_hex("010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b0201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0804ffff001d026e04ffffffff0100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bdef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99c48b0e3f39c51347a19a5819651503a5ac00000000010000000321f75f3139a013f50f315b23b0c9a2b6eac31e2bec98e5891c924664889942260000000049483045022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067fa6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a5014081b5c01ffffffff79cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b84d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea5982f88cef651e9354f5791602369bf5a82a6cd61a62501fffffffffe09f5fe3ffbf5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e82000000004847304402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d33812a5ce01ffffffff01009d966b01000000434104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac00000000").unwrap();

        let prevhash =
            Vec::<u8>::from_hex("4ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000")
                .unwrap();
        let merkle =
            Vec::<u8>::from_hex("bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914c")
                .unwrap();

        let origin_block: bitcoin::Block = deserialize(&block_bytes).unwrap();
        let origin_block_header = origin_block.header;
        let origin_block_header_bytes = serialize(&origin_block_header);
        let block: Block = origin_block.clone().into();
        let exp_recipient_addresses: Vec<BitcoinAddress> = vec![
            "1J58xY6MTsp5r9CwWqVsq8syqxiZxJBAKm".parse().unwrap(),
            "1BBz9Z15YpELQ4QP5sEKb1SwxkcmPb5TMs".parse().unwrap(),
        ];
        let recipient_addresses: Vec<BitcoinAddress> = block
            .clone()
            .txdata
            .into_iter()
            .flat_map(|tx| tx.output.into_iter().map(|tx_out| tx_out.recipient_address))
            .collect();
        assert_eq!(exp_recipient_addresses, recipient_addresses);
        let block_bcs_hex = hex::encode(bcs::to_bytes(&block).unwrap());
        //make sure the block bcs encode not changed, and we also use this hex to test in move
        assert_eq!(block_bcs_hex.as_str(), "010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b0221da2ae8cc773b020b4873f597369416cf961a1896c24106b0198459fec2df770100000000000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0804ffff001d026e04ffffffff000100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bdef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99c48b0e3f39c51347a19a5819651503a5ac1500bb4271dca9fae6473f4866b5982936f90ebd2abc339d9a371e2b5a26147ddfd87228b900ff75762a18a40f2778bedbcde7e9b0a301000000000000000321f75f3139a013f50f315b23b0c9a2b6eac31e2bec98e5891c924664889942260000000049483045022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067fa6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a5014081b5c01ffffffff0079cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b84d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea5982f88cef651e9354f5791602369bf5a82a6cd61a62501ffffffff00fe09f5fe3ffbf5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e82000000004847304402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d33812a5ce01ffffffff0001009d966b01000000434104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac15006fc51f2a519d341392c2231faec5e91881250b5a");

        let block_header: Header = origin_block_header.into();

        assert_eq!(block_header.version, 1);
        assert_eq!(block_header.prev_blockhash.to_vec(), prevhash);
        assert_eq!(
            block_header.merkle_root,
            origin_block.compute_merkle_root().unwrap().into_address()
        );
        assert_eq!(block_header.merkle_root.to_vec(), merkle);
        assert_eq!(block_header.time, 1231965655);
        assert_eq!(block_header.bits, 486604799);
        assert_eq!(block_header.nonce, 2067413810);

        assert_eq!(block_header.encode(), origin_block_header_bytes);
        assert_eq!(
            block_header.block_hash(),
            origin_block_header.block_hash().into_address()
        );
        assert!(block.txdata[0].is_coinbase());
    }

    #[test]
    fn test_block_hash() {
        //https://mempool.space/block/00000000000000000002b73f69e81b8b5e98dff0f2b7632fcb83c050c3b099a1
        let version = 536879108;
        //bitcoin hex is reversed
        let prev_blockhash = bitcoin::BlockHash::from_str(
            "00000000000000000009d54a110cc122960d31567d3ee84a1f18a98f50591046",
        )
        .unwrap();
        let merkle_root = bitcoin::TxMerkleNode::from_str(
            "e1e0573e6098d8128ee859e7540f56b01fe0a33e56694df6d2fab0f96c4954b3",
        )
        .unwrap();

        let time = 1644403033;
        let bits = 0x170a8bb4;
        let nonce = 1693537958;
        let header = Header::new(
            version,
            prev_blockhash.into_address(),
            merkle_root.into_address(),
            time,
            bits,
            nonce,
        );

        let origin_header = bitcoin::block::Header {
            version: Version::from_consensus(version as i32),
            prev_blockhash,
            merkle_root,
            time,
            bits: CompactTarget::from_consensus(bits),
            nonce,
        };

        assert_eq!(header.encode(), serialize(&origin_header));

        let hash = header.block_hash();
        let expect_block_hash =
            BlockHash::from_str("00000000000000000002b73f69e81b8b5e98dff0f2b7632fcb83c050c3b099a1")
                .unwrap();
        assert_eq!(hash, expect_block_hash.into_address());

        assert_eq!(origin_header.block_hash(), expect_block_hash);
    }

    #[test]
    fn test_coin_base_tx() {
        //https://mempool.space/api/tx/3ea07d9966895a8a73a5580d34713b8ff302a8413215af156e2ad484e50ccc5c/hex
        let tx_bytes = Vec::<u8>::from_hex("010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff56035cea0c194d696e656420627920416e74506f6f6c20b9004206d7a9abb4fabe6d6dbbd991d69c05a27bd76b9bc7ad80763da6d836be289c7a53e12612625d5d1fec100000000000000000003d64a66d000000000000ffffffff05220200000000000017a91442402a28dd61f2718a4b27ae72a4791d5bbdade7872d04b0130000000017a9145249bdf2c131d43995cff42e8feee293f79297a8870000000000000000266a24aa21a9ede27dc3f39ba542af6f3b7b10d1b36d123910d46438a360e718ffcdd550d3c37e00000000000000002f6a2d434f52450142fdeae88682a965939fee9b7b2bd5b99694ff644e3ecda72cb7961caa4b541b1e322bcfe0b5a03000000000000000002b6a2952534b424c4f434b3a920ea155edd52e4efb952d4cec821261746fb0aa72b2c1552c1cce2b0061b56e0120000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let bitcoin_tx: bitcoin::Transaction = deserialize(&tx_bytes).unwrap();
        assert!(bitcoin_tx.is_coinbase());
        let tx: Transaction = bitcoin_tx.into();
        assert!(tx.is_coinbase());
    }

    #[test]
    fn test_from_bitcoin_tx_out() {
        // p2pk script(outpoint: e1be133be54851d21f34666ae45211d6e76d60491cecfef17bba90731eb8f42a:0)
        let tx_out = bitcoin::TxOut{
            value: Amount::from_sat(5000000000),
            script_pubkey: bitcoin::ScriptBuf::from_hex("4104f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9ac").unwrap(),
        };
        assert!(tx_out.script_pubkey.is_p2pk());
        let tx_out_rooch = TxOut::from(tx_out.clone());
        assert_eq!(
            "1DR5CqnzFLDmPZ7h94RHTxLV7u19xkS5rn",
            tx_out_rooch.recipient_address.to_string()
        );
        // p2ms script(outpoint: a353a7943a2b38318bf458b6af878b8384f48a6d10aad5b827d0550980abe3f0:0)
        let tx_out = bitcoin::TxOut {
            value: Amount::from_sat(5000000000),
            script_pubkey: bitcoin::ScriptBuf::from_hex(
                "0014f29f9316f0f1e48116958216a8babd353b491dae",
            )
            .unwrap(),
        };
        let tx_out_rooch = TxOut::from(tx_out.clone());
        assert_eq!(
            "bc1q720ex9hs78jgz954sgt23w4ax5a5j8dwjj5kkm",
            tx_out_rooch.recipient_address.to_string()
        );
    }
}
