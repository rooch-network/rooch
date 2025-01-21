// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{
    register_histogram_vec_with_registry, register_int_gauge_with_registry, HistogramVec, IntGauge,
    Registry,
};

#[derive(Debug)]
pub struct ProposerMetrics {
    pub proposer_propose_block_latency_seconds: HistogramVec,
    pub proposer_propose_block_batch_size: IntGauge,
}

impl ProposerMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        ProposerMetrics {
            proposer_propose_block_latency_seconds: register_histogram_vec_with_registry!(
                "proposer_propose_block_latency_seconds",
                "Proposer propose block latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            proposer_propose_block_batch_size: register_int_gauge_with_registry!(
                "proposer_propose_block_batch_size",
                "Proposer propose block contains how many transactions",
                registry,
            )
            .unwrap(),
        }
    }
}
