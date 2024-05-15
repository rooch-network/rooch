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
