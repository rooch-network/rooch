// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::server::logger::Logger;
use tracing::Level;

#[derive(Debug, Clone)]
pub struct RpcLogger;

impl Logger for RpcLogger {
    type Instant = std::time::Instant;

    fn on_connect(
        &self,
        _remote_addr: std::net::SocketAddr,
        _request: &jsonrpsee::server::logger::HttpRequest,
        _t: jsonrpsee::server::logger::TransportProtocol,
    ) {
    }

    fn on_request(
        &self,
        _transport: jsonrpsee::server::logger::TransportProtocol,
    ) -> Self::Instant {
        std::time::Instant::now()
    }

    fn on_call(
        &self,
        method_name: &str,
        params: jsonrpsee::types::Params,
        _kind: jsonrpsee::server::logger::MethodKind,
        transport: jsonrpsee::server::logger::TransportProtocol,
    ) {
        //TODO remove param parse when server stable.
        let params_str = match params.parse::<serde_json::Value>() {
            Ok(json) => json.to_string(),
            Err(e) => e.to_string(),
        };
        tracing::event!(
            Level::INFO,
            event = "on_call",
            transport = transport.to_string(),
            method_name = method_name,
            params = params_str,
        );
    }

    fn on_result(
        &self,
        method_name: &str,
        success: bool,
        started_at: Self::Instant,
        _transport: jsonrpsee::server::logger::TransportProtocol,
    ) {
        let elapsed_millis = started_at.elapsed().as_millis();
        tracing::event!(
            Level::INFO,
            event = "on_result",
            method_name = method_name,
            success = success,
            elapsed_millis = elapsed_millis
        );
    }

    fn on_response(
        &self,
        _result: &str,
        _started_at: Self::Instant,
        _transport: jsonrpsee::server::logger::TransportProtocol,
    ) {
    }

    fn on_disconnect(
        &self,
        _remote_addr: std::net::SocketAddr,
        _transport: jsonrpsee::server::logger::TransportProtocol,
    ) {
    }
}
