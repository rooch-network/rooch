// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::{
    absolute::LockTime, consensus::deserialize, hex::FromHex, transaction::Version, Amount, Block,
    Sequence, Transaction, Txid, Witness,
};
use include_dir::{include_dir, Dir};
use rooch_types::bitcoin::network::Network;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Download the bitcoin block via the following command:
// curl -sSL "https://mempool.space/api/block/00000000000af0aed4792b1acee3d966af36cf5def14935db8de83d6f9306f2f/raw" > crates/rooch-framework-tests/blocks/bitcoin/91812.blob
// curl -sSL "https://mempool.space/api/block/00000000000a4d0a398161ffc163c503763b1f4360639393e0e4c8e300e0caec/raw" > crates/rooch-framework-tests/blocks/bitcoin/91842.blob
// curl -sSL "https://mempool.space/api/block/000000000000000000020750f322f4e72e99c2f0b9738fb4f46607860bd18c13/raw" > crates/rooch-framework-tests/blocks/bitcoin/818677.blob
// curl -sSL "https://mempool.space/testnet/api/block/0000000016412abe1778a347da773ff8bc087ad1a91ae5daad349bc268285c2d/raw" > crates/rooch-framework-tests/blocks/testnet/2821527.blob
pub(crate) const STATIC_BLOCK_DIR: Dir = include_dir!("blocks");

pub(crate) fn load_block(network: Network, height: u64) -> Block {
    let block_file = PathBuf::from(network.to_string()).join(format!("{}.blob", height));
    let btc_block_bytes = STATIC_BLOCK_DIR
        .get_file(block_file.as_path())
        .unwrap()
        .contents();
    let block: Block = deserialize(btc_block_bytes).unwrap();
    block
}

pub(crate) fn load_tx(network: Network, txid: &str) -> Transaction {
    let tx_file = PathBuf::from(network.to_string()).join(format!("{}.txt", txid));
    let btc_tx_hex = STATIC_BLOCK_DIR
        .get_file(tx_file.as_path())
        .unwrap()
        .contents_utf8()
        .unwrap();
    let tx: Transaction = bitcoin_tx_from_hex(btc_tx_hex);
    tx
}

pub(crate) fn load_tx_info(network: Network, txid: &str) -> TxInfo {
    let tx_file = PathBuf::from(network.to_string()).join(format!("{}.json", txid));
    let btc_tx_json = STATIC_BLOCK_DIR
        .get_file(tx_file.as_path())
        .unwrap()
        .contents_utf8()
        .unwrap();
    let tx: TxInfo = serde_json::from_str(btc_tx_json).unwrap();
    tx
}

pub(crate) fn bitcoin_tx_from_hex(hex: &str) -> Transaction {
    let btc_tx_bytes = Vec::from_hex(hex).unwrap();
    let btc_tx: Transaction = deserialize(btc_tx_bytes.as_slice()).unwrap();
    btc_tx
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: String,
    pub value: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInputInfo {
    pub txid: Txid,
    pub vout: u32,
    pub prevout: OutputInfo,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    pub witness: Vec<String>,
    pub is_coinbase: bool,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInfo {
    pub txid: Txid,
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<TxInputInfo>,
    pub vout: Vec<OutputInfo>,
}

impl From<TxInfo> for Transaction {
    fn from(tx_info: TxInfo) -> Self {
        let mut tx = Transaction {
            version: Version::non_standard(tx_info.version),
            lock_time: LockTime::from_consensus(tx_info.locktime),
            input: vec![],
            output: vec![],
        };
        for input in tx_info.vin {
            let txid = input.txid;
            let vout = input.vout;

            let script_sig = input.scriptsig;
            let sequence = input.sequence;
            let witness = input
                .witness
                .into_iter()
                .map(|w| Vec::from_hex(&w).unwrap())
                .collect::<Vec<Vec<u8>>>();
            let tx_input = bitcoin::TxIn {
                previous_output: bitcoin::OutPoint { txid, vout },
                script_sig: bitcoin::ScriptBuf::from_hex(&script_sig).unwrap(),
                sequence: Sequence::from_consensus(sequence),
                witness: Witness::from_slice(&witness),
            };
            tx.input.push(tx_input);
        }
        for output in tx_info.vout {
            let script_pubkey = output.scriptpubkey;
            let value = output.value;
            let tx_output = bitcoin::TxOut {
                script_pubkey: bitcoin::ScriptBuf::from_hex(&script_pubkey).unwrap(),
                value,
            };
            tx.output.push(tx_output);
        }
        tx
    }
}
