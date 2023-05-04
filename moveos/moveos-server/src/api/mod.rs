// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod account;

use jsonrpsee::RpcModule;

/// To avoid unnecessary dependency on that crate, we have a reference here
/// for document purposes.
pub const QUERY_MAX_RESULT_LIMIT: usize = 1000;
// TODOD(chris): make this configurable
pub const QUERY_MAX_RESULT_LIMIT_CHECKPOINTS: usize = 100;

pub const QUERY_MAX_RESULT_LIMIT_OBJECTS: usize = 256;
//
// pub fn cap_page_limit(limit: Option<usize>) -> usize {
//     let limit = limit.unwrap_or_default();
//     if limit > QUERY_MAX_RESULT_LIMIT || limit == 0 {
//         QUERY_MAX_RESULT_LIMIT
//     } else {
//         limit
//     }
// }
//
// pub fn validate_limit(limit: Option<usize>, max: usize) -> Result<usize, anyhow::Error> {
//     match limit {
//         Some(l) if l > max => Err(anyhow!("Page size limit {l} exceeds max limit {max}")),
//         Some(0) => Err(anyhow!("Page size limit cannot be smaller than 1")),
//         Some(l) => Ok(l),
//         None => Ok(max),
//     }
// }

pub trait RoochRpcModule
    where
        Self: Sized,
{
    fn rpc(self) -> RpcModule<Self>;
}
