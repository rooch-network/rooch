// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::H256View;
use moveos_types::h256::H256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct StateOptions {
    /// If true, the state is decoded and the decoded value is returned in the response.
    pub decode: bool,
    /// If true, result with display rendered is returned
    pub show_display: bool,
    /// The state root of remote stateDB
    pub state_root: Option<H256View>,
}

impl StateOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decode(mut self, decode: bool) -> Self {
        self.decode = decode;
        self
    }

    pub fn show_display(mut self, show_display: bool) -> Self {
        self.show_display = show_display;
        self
    }

    pub fn state_root(mut self, state_root: Option<H256>) -> Self {
        match state_root {
            None => {}
            Some(h256) => {
                self.state_root = Some(H256View::from(h256));
            }
        }
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TxOptions {
    /// If true, the TransactionOutput is returned in the response.
    pub with_output: bool,
    /// If true, the event is decoded and the decoded value is returned in the response.
    /// Only valid when with_output is true.
    pub decode: bool,
}

impl TxOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_output(mut self, with_output: bool) -> Self {
        self.with_output = with_output;
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Eq, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueryOptions {
    /// If true, return query items in descending order.
    pub descending: bool,
    /// If true, the state is decoded and the decoded value is returned in the response.
    pub decode: bool,
    /// If true, result with display rendered is returned
    pub show_display: bool,
    /// If true, filter out all match items.
    pub filter_out: bool,
}

impl QueryOptions {
    pub fn descending(mut self, descending: bool) -> Self {
        self.descending = descending;
        self
    }

    pub fn show_display(mut self, show_display: bool) -> Self {
        self.show_display = show_display;
        self
    }

    pub fn filter_out(mut self, filter_out: bool) -> Self {
        self.filter_out = filter_out;
        self
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            descending: true,
            decode: false,
            show_display: false,
            filter_out: false,
        }
    }
}
