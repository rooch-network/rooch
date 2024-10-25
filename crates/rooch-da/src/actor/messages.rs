// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use rooch_types::da::batch::SignedDABatchMeta;
use rooch_types::da::state::ServerStatus;
use rooch_types::transaction::LedgerTransaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PutDABatchMessage {
    pub tx_order_start: u64,
    pub tx_order_end: u64,
    pub tx_list: Vec<LedgerTransaction>,
}

impl Message for PutDABatchMessage {
    type Result = anyhow::Result<SignedDABatchMeta>;
}

impl PutDABatchMessage {
    pub fn new(tx_order_start: u64, tx_order_end: u64, tx_list: Vec<LedgerTransaction>) -> Self {
        Self {
            tx_order_start,
            tx_order_end,
            tx_list,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServerStatusMessage {}

impl Message for GetServerStatusMessage {
    type Result = anyhow::Result<ServerStatus>;
}
