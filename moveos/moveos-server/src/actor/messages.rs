// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HelloMessage {
    pub msg: String,
}

impl Message for HelloMessage {
    type Result = String;
}
