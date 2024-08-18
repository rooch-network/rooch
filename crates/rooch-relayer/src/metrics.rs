// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct TxMetrics {
    pub tx_execution_latency_seconds: HistogramVec,
    pub tx_execution_bytes: HistogramVec,
}

impl TxMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        TxMetrics {
            tx_execution_latency_seconds: register_histogram_vec_with_registry!(
                "tx_execution_latency_seconds",
                "Tx execution latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            tx_execution_bytes: register_histogram_vec_with_registry!(
                "tx_execution_bytes",
                "Tx execution size in bytes",
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
