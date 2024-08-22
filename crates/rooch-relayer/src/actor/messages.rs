// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoin::Transaction;
use bitcoincore_rpc::bitcoin::Txid;
use bitcoincore_rpc::json;
use coerce::actor::{message::Message, scheduler::timer::TimerTick};
use rooch_types::transaction::{L1BlockWithBody, L1Transaction};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RelayTick {}

impl Message for RelayTick {
    type Result = ();
}

impl TimerTick for RelayTick {}

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

pub struct GetReadyL1BlockMessage {}

impl Message for GetReadyL1BlockMessage {
    type Result = Result<Option<L1BlockWithBody>>;
}

pub struct GetReadyL1TxsMessage {}

impl Message for GetReadyL1TxsMessage {
    type Result = Result<Vec<L1Transaction>>;
}

#[derive(Clone)]
pub struct SyncTick {}

impl Message for SyncTick {
    type Result = ();
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
