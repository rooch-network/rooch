// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct ExecutorMetrics {
    pub executor_execute_tx_latency_seconds: HistogramVec,
    pub executor_execute_tx_bytes: HistogramVec,
    pub executor_validate_tx_latency_seconds: HistogramVec,
    pub executor_validate_tx_bytes: HistogramVec,
}

impl ExecutorMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        ExecutorMetrics {
            executor_execute_tx_latency_seconds: register_histogram_vec_with_registry!(
                "executor_execute_tx_latency_seconds",
                "Executor execute tx latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            executor_execute_tx_bytes: register_histogram_vec_with_registry!(
                "executor_execute_tx_bytes",
                "Executor execute tx size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            executor_validate_tx_latency_seconds: register_histogram_vec_with_registry!(
                "executor_validate_tx_latency_seconds",
                "Executor validate tx latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            executor_validate_tx_bytes: register_histogram_vec_with_registry!(
                "executor_validate_tx_bytes",
                "Executor validate tx size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
        }
    }
}
