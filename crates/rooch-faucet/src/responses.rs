// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::FaucetError;
use anyhow::Result;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultResponse<V> {
    pub ok: Option<V>,
    pub error: Option<String>,
}

impl<T, V> From<Result<T, FaucetError>> for ResultResponse<V>
where
    V: From<T>,
{
    fn from(result: Result<T, FaucetError>) -> Self {
        match result {
            Ok(t) => Self {
                ok: Some(t.into()),
                error: None,
            },
            Err(e) => Self {
                ok: None,
                error: Some(e.to_string()),
            },
        }
    }
}

impl<V> IntoResponse for ResultResponse<V>
where
    V: Serialize,
{
    fn into_response(self) -> Response {
        let status = if self.error.is_some() {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::OK
        };
        (status, axum::Json(self)).into_response()
    }
}
