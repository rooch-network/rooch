// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use rooch_types::da::state::DAServerStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServerStatusMessage {}

impl Message for GetServerStatusMessage {
    type Result = anyhow::Result<DAServerStatus>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppendTransactionMessage {
    pub tx_order: u64,
    pub tx_timestamp: u64,
}

impl Message for AppendTransactionMessage {
    type Result = anyhow::Result<()>;
}

impl AppendTransactionMessage {
    pub fn new(tx_order: u64, tx_timestamp: u64) -> Self {
        Self {
            tx_order,
            tx_timestamp,
        }
    }
}
