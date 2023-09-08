// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::RpcModule;

pub mod eth_api;
pub mod rooch_api;
pub mod wallet_api;

pub const DEFAULT_RESULT_LIMIT: u64 = 50;
pub const DEFAULT_RESULT_LIMIT_USIZE: usize = DEFAULT_RESULT_LIMIT as usize;

pub const MAX_RESULT_LIMIT: u64 = 200;
pub const MAX_RESULT_LIMIT_USIZE: usize = MAX_RESULT_LIMIT as usize;

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
