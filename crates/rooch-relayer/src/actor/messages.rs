// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use coerce::actor::scheduler::timer::TimerTick;
use rooch_types::transaction::{L1BlockWithBody, L1Transaction};

#[derive(Clone)]
pub struct RelayTick {}

impl Message for RelayTick {
    type Result = ();
}

impl TimerTick for RelayTick {}

#[derive(Clone)]
pub struct SyncTick {}

impl Message for SyncTick {
    type Result = ();
}

pub struct GetReadyL1BlockMessage {}

impl Message for GetReadyL1BlockMessage {
    type Result = anyhow::Result<Option<L1BlockWithBody>>;
}

pub struct GetReadyL1TxsMessage {}

impl Message for GetReadyL1TxsMessage {
    type Result = anyhow::Result<Vec<L1Transaction>>;
}
