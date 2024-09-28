// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::FaucetError;
use move_core_types::u256::U256;
use rooch_rpc_api::jsonrpc_types::StrView;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FaucetResponse {
    pub gas: StrView<U256>,
    pub error: Option<String>,
}

impl From<FaucetError> for FaucetResponse {
    fn from(e: FaucetError) -> Self {
        Self {
            gas: StrView(U256::zero()),
            error: Some(e.to_string()),
        }
    }
}

impl From<U256> for FaucetResponse {
    fn from(gas: U256) -> Self {
        Self {
            gas: StrView(gas),
            error: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfoResponse {
    pub gas_balance: StrView<U256>,
    pub error: Option<String>,
}

impl From<FaucetError> for InfoResponse {
    fn from(e: FaucetError) -> Self {
        Self {
            error: Some(e.to_string()),
            gas_balance: StrView(U256::zero()),
        }
    }
}

impl From<U256> for InfoResponse {
    fn from(v: U256) -> Self {
        Self {
            error: None,
            gas_balance: StrView(v),
        }
    }
}
