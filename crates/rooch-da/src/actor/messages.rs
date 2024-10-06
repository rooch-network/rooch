// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use rooch_types::da::batch::DABatch;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PutDABatchMessage {
    pub batch: DABatch,
}

impl Message for PutDABatchMessage {
    type Result = anyhow::Result<()>;
}

pub struct Shit {
    pub a: DABatch,
}

impl Message for Shit {
    type Result = anyhow::Result<()>;
}
