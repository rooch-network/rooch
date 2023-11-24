// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{addresses::ROOCH_FRAMEWORK_ADDRESS, into_address::IntoAddress};
use anyhow::Result;
use bitcoin::{consensus::Encodable, hashes::Hash, BlockHash};
use bitcoincore_rpc::bitcoincore_rpc_json::GetBlockHeaderResult;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("bitcoin_types");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// The block header
    pub header: Header,
    /// List of transactions contained in the block
    pub txdata: Vec<Transaction>,
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
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
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
    pub witness: Vec<u8>,
}

impl From<bitcoin::TxIn> for TxIn {
    fn from(tx_in: bitcoin::TxIn) -> Self {
        let mut witness = vec![];
        tx_in
            .witness
            .consensus_encode(&mut witness)
            .expect("encode witness to byte array should success");
        Self {
            previous_output: tx_in.previous_output.into(),
            script_sig: tx_in.script_sig.into_bytes(),
            sequence: tx_in.sequence.0,
            witness,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutPoint {
    /// The referenced transaction's txid.
    /// Use address to represent sha256d hash
    pub txid: AccountAddress,
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
}

impl From<bitcoin::OutPoint> for OutPoint {
    fn from(out_point: bitcoin::OutPoint) -> Self {
        Self {
            txid: out_point.txid.into_address(),
            vout: out_point.vout,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptBuf{
    pub bytes: Vec<u8>,
}

impl From<bitcoin::ScriptBuf> for ScriptBuf {
    fn from(script: bitcoin::ScriptBuf) -> Self {
        Self {
            bytes: script.into_bytes(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxOut {
    /// The value of the output, in satoshis.
    pub value: u64,
    /// The script which must be satisfied for the output to be spent.
    pub script_pubkey: ScriptBuf,
}

impl From<bitcoin::TxOut> for TxOut {
    fn from(tx_out: bitcoin::TxOut) -> Self {
        Self {
            value: tx_out.value.to_sat(),
            script_pubkey: tx_out.script_pubkey.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{consensus::deserialize, Block};
    use hex::FromHex;

    #[test]
    fn test_header() {
        // Mainnet block 00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7
        let some_block = Vec::<u8>::from_hex("010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b0201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0804ffff001d026e04ffffffff0100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bdef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99c48b0e3f39c51347a19a5819651503a5ac00000000010000000321f75f3139a013f50f315b23b0c9a2b6eac31e2bec98e5891c924664889942260000000049483045022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067fa6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a5014081b5c01ffffffff79cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b84d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea5982f88cef651e9354f5791602369bf5a82a6cd61a62501fffffffffe09f5fe3ffbf5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e82000000004847304402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d33812a5ce01ffffffff01009d966b01000000434104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac00000000").unwrap();

        let prevhash =
            Vec::<u8>::from_hex("4ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000")
                .unwrap();
        let merkle =
            Vec::<u8>::from_hex("bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914c")
                .unwrap();

        let decode: Block = deserialize(&some_block).unwrap();
        let block_header: Header = decode.header.into();

        assert_eq!(block_header.version, 1);
        assert_eq!(block_header.prev_blockhash.to_vec(), prevhash);
        assert_eq!(
            block_header.merkle_root,
            decode.compute_merkle_root().unwrap().into_address()
        );
        assert_eq!(block_header.merkle_root.to_vec(), merkle);
        assert_eq!(block_header.time, 1231965655);
        assert_eq!(block_header.bits, 486604799);
        assert_eq!(block_header.nonce, 2067413810);
    }
}
