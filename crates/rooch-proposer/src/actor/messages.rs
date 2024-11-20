// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::{message::Message, scheduler::timer::TimerTick};

#[derive(Clone)]
pub struct ProposeBlock {}

impl Message for ProposeBlock {
    type Result = ();
}

impl TimerTick for ProposeBlock {}
