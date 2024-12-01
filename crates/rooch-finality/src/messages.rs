// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use rooch_types::finality_block::Block;
use serde::{Deserialize, Serialize};

/// Finality Message
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalityMessage {
    pub block: Block,
}

impl Message for FinalityMessage {
    type Result = Result<()>;
}
