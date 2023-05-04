// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod account;

use jsonrpsee::RpcModule;

pub const MAX_RESULT_LIMIT: usize = 1000;

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
