// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoin::Transaction;
use bitcoincore_rpc::bitcoin::Txid;
use bitcoincore_rpc::json;
use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockMessage {
    pub hash: bitcoin::BlockHash,
}

impl Message for GetBlockMessage {
    type Result = Result<bitcoin::Block>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBestBlockHashMessage {}

impl Message for GetBestBlockHashMessage {
    type Result = Result<bitcoin::BlockHash>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockHashMessage {
    pub height: u64,
}

impl Message for GetBlockHashMessage {
    type Result = Result<bitcoin::BlockHash>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockHeaderInfoMessage {
    pub hash: bitcoin::BlockHash,
}

impl Message for GetBlockHeaderInfoMessage {
    type Result = Result<json::GetBlockHeaderResult>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetChainTipsMessage {}

impl Message for GetChainTipsMessage {
    type Result = Result<json::GetChainTipsResult>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastTransactionMessage {
    pub hex: String,
    pub maxfeerate: Option<f64>,
    pub maxburnamount: Option<f64>,
}

impl Message for BroadcastTransactionMessage {
    type Result = Result<Txid>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxOutMessage {
    pub txid: Txid,
    pub vout: u32,
    pub include_mempool: Option<bool>,
}

impl GetTxOutMessage {
    pub fn new(txid: Txid, vout: u32) -> Self {
        Self {
            txid,
            vout,
            include_mempool: Some(false),
        }
    }
}

impl Message for GetTxOutMessage {
    type Result = Result<Option<json::GetTxOutResult>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRawTransactionMessage {
    pub txid: Txid,
}

impl Message for GetRawTransactionMessage {
    type Result = Result<Transaction>;
}
