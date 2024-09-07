// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct SequencerMetrics {
    pub sequencer_sequence_latency_seconds: HistogramVec,
    pub sequencer_sequence_bytes: HistogramVec,
}

impl SequencerMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        SequencerMetrics {
            sequencer_sequence_latency_seconds: register_histogram_vec_with_registry!(
                "sequencer_sequence_latency_seconds",
                "Sequencer sequence latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            sequencer_sequence_bytes: register_histogram_vec_with_registry!(
                "sequencer_sequence_bytes",
                "Sequencer sequence size in bytes",
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
