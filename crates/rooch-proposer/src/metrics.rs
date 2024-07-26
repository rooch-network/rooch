// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct ProposerMetrics {
    pub proposer_transaction_propose_latency_seconds: HistogramVec,
    pub proposer_transaction_propose_bytes: HistogramVec,
    pub proposer_propose_block_latency_seconds: HistogramVec,
    pub proposer_propose_block_bytes: HistogramVec,
}

impl ProposerMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        ProposerMetrics {
            proposer_transaction_propose_latency_seconds: register_histogram_vec_with_registry!(
                "proposer_transaction_propose_latency_seconds",
                "Proposer transaction propose latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            proposer_transaction_propose_bytes: register_histogram_vec_with_registry!(
                "proposer_transaction_propose_bytes",
                "Proposer transaction propose size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            proposer_propose_block_latency_seconds: register_histogram_vec_with_registry!(
                "proposer_propose_block_latency_seconds",
                "Proposer propose block latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            proposer_propose_block_bytes: register_histogram_vec_with_registry!(
                "proposer_propose_block_bytes",
                "Proposer propose block size in bytes",
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
