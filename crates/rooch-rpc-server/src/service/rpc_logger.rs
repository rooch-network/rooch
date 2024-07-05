// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::server::middleware::rpc::RpcServiceT;
use jsonrpsee::types::Request;

#[derive(Clone)]
pub struct RpcLogger<S>(pub S);

impl<'a, S> RpcServiceT<'a> for RpcLogger<S>
where
    S: RpcServiceT<'a> + Send + Sync,
{
    type Future = S::Future;

    fn call(&self, req: Request<'a>) -> Self::Future {
        let params_str = match req.params().parse::<serde_json::Value>() {
            Ok(json) => json.to_string(),
            Err(e) => e.to_string(),
        };

        tracing::event!(
            tracing::Level::INFO,
            event = "on_call",
            method_name = req.method_name(),
            params = params_str,
        );

        self.0.call(req)
    }
}
