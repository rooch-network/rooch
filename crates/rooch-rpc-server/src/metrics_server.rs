// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use prometheus::Registry;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};

pub const METRICS_HOST_PORT: u16 = 9184;

/// This is an option if you need to use the underlying method
pub use metrics::start_prometheus_server;
use raw_store::metrics::DBMetrics;
use rooch_indexer::store::metrics::IndexerDBMetrics;

/// Use the standard IP (0.0.0.0) and port (9184) to start a new
/// prometheus server.
///
/// Use this function to get a registry and then register your
/// own application metrics via
///
/// ```
/// use prometheus::{register_int_counter_with_registry, IntCounter, Registry};
/// use rooch_rpc_server::metrics_server::start_basic_prometheus_server;
///
/// pub struct MyMetrics {
///     pub requests: IntCounter,
/// }
///
/// impl MyMetrics {
///     pub fn new(registry: &Registry) -> Self {
///         Self {
///             requests: register_int_counter_with_registry!(
///                 "total_requests",
///                 "Total number of requests received by my service",
///                 registry
///             )
///             .unwrap(),
///         }
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let prometheus_registry = start_basic_prometheus_server();
///     let metrics = MyMetrics::new(&prometheus_registry);
/// }
/// ```
pub fn start_basic_prometheus_server() -> Registry {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), METRICS_HOST_PORT);
    start_prometheus_server(addr).default_registry()
}

pub fn init_metrics(prometheus_registry: &Registry) {
    DBMetrics::init(prometheus_registry);
    IndexerDBMetrics::init(prometheus_registry);
}
