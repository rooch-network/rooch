// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::FaucetError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FaucetResponse {
    pub transferred_gas_objects: Vec<String>,
    pub error: Option<String>,
}

impl From<FaucetError> for FaucetResponse {
    fn from(e: FaucetError) -> Self {
        Self {
            error: Some(e.to_string()),
            transferred_gas_objects: vec![],
        }
    }
}

impl From<String> for FaucetResponse {
    fn from(v: String) -> Self {
        Self {
            transferred_gas_objects: vec![v],
            error: None,
        }
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
