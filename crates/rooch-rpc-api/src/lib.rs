// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::non_canonical_clone_impl)]

pub mod api;
pub mod jsonrpc_types;

pub type RpcResult<T> = Result<T, RpcError>;
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use rooch_types::error::RoochError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error(transparent)]
    RoochError(#[from] RoochError),

    #[error(transparent)]
    InternalError(#[from] anyhow::Error),

    #[error("Deserialization error: {0}")]
    BcsError(#[from] bcs::Error),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::RoochError(err) => ErrorObject::owned(1, err.to_string(), None::<()>),
            RpcError::InternalError(err) => ErrorObject::owned(2, err.to_string(), None::<()>),
            RpcError::BcsError(err) => ErrorObject::owned(3, err.to_string(), None::<()>),
            RpcError::UnexpectedError(err) => ErrorObject::owned(4, err.to_string(), None::<()>),
        }
    }
}
