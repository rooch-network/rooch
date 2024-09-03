// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::BytesView;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountSignView {
    pub msg: String,
    pub payload: AuthPayloadView,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthPayloadView {
    pub signature: BytesView,
    pub message_prefix: BytesView,
    pub message_info: BytesView,
    pub public_key: BytesView,
    pub from_address: String,
}

impl AccountSignView {
    pub fn new(msg: String, payload: AuthPayloadView) -> Self {
        Self { msg, payload }
    }
}

impl AuthPayloadView {
    pub fn new(
        signature: BytesView,
        message_prefix: BytesView,
        message_info: BytesView,
        public_key: BytesView,
        from_address: String,
    ) -> Self {
        Self {
            signature,
            message_prefix,
            message_info,
            public_key,
            from_address,
        }
    }
}
