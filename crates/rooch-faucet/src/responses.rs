// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::FaucetError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FaucetResponse {
    pub gas: String,
    pub error: Option<String>,
}

impl From<FaucetError> for FaucetResponse {
    fn from(e: FaucetError) -> Self {
        Self {
            gas: "0".to_string(),
            error: Some(e.to_string()),
        }
    }
}

impl From<String> for FaucetResponse {
    fn from(gas: String) -> Self {
        Self { gas, error: None }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InfoResponse {
    pub gas_balance: f64,
    pub error: Option<String>,
}

impl From<FaucetError> for InfoResponse {
    fn from(e: FaucetError) -> Self {
        Self {
            error: Some(e.to_string()),
            gas_balance: 0f64,
        }
    }
}

impl From<f64> for InfoResponse {
    fn from(v: f64) -> Self {
        Self {
            error: None,
            gas_balance: v,
        }
    }
}
