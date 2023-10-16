// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct StateOptions {
    /// If true, the state is decoded and the decoded value is returned in the response.
    pub decode: bool,
}

impl StateOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decode(mut self, decode: bool) -> Self {
        self.decode = decode;
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EventOptions {
    /// If true, the event is decoded and the decoded value is returned in the response.
    pub decode: bool,
}

impl EventOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decode(mut self, decode: bool) -> Self {
        self.decode = decode;
        self
    }
}
