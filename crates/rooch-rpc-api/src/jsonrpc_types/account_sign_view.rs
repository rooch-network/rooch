// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::{crypto::Signature, framework::auth_payload::AuthPayload};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountSignView {
    pub signature: Signature,
    pub payload: AuthPayload,
}

impl AccountSignView {
    pub fn new(signature: Signature, payload: AuthPayload) -> Self {
        Self { signature, payload }
    }
}
