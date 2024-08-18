// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountSignView {
    pub msg: String,
    pub signature: String,
}

impl AccountSignView {
    pub fn new(msg: String, signature: String) -> Self {
        Self { msg, signature }
    }
}
