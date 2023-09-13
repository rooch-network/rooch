// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::{message::Message, scheduler::timer::TimerTick};

#[derive(Clone)]
pub struct RelayTick {}

impl Message for RelayTick {
    type Result = ();
}

impl TimerTick for RelayTick {}
