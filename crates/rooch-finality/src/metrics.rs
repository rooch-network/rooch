// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct FinalityMetrics {
    pub finality_latency_seconds: HistogramVec,
    pub finality_bytes: HistogramVec,
}

impl FinalityMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        FinalityMetrics {
            finality_latency_seconds: register_histogram_vec_with_registry!(
                "finality_latency_seconds",
                "Finality sequence latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            finality_bytes: register_histogram_vec_with_registry!(
                "finality_bytes",
                "Finality sequence size in bytes",
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
