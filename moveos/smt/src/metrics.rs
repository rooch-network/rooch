// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct SMTMetrics {
    pub smt_put_latency_seconds: HistogramVec,
    pub smt_put_bytes: HistogramVec,
    pub smt_remove_latency_seconds: HistogramVec,
    pub smt_remove_bytes: HistogramVec,
    pub smt_get_latency_seconds: HistogramVec,
    pub smt_get_bytes: HistogramVec,
    pub smt_get_with_proof_latency_seconds: HistogramVec,
    pub smt_get_with_proof_bytes: HistogramVec,
    pub smt_list_latency_seconds: HistogramVec,
    pub smt_list_bytes: HistogramVec,
    pub smt_iter_latency_seconds: HistogramVec,
    pub smt_puts_latency_seconds: HistogramVec,
    pub smt_puts_bytes: HistogramVec,
}

impl SMTMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        SMTMetrics {
            smt_put_latency_seconds: register_histogram_vec_with_registry!(
                "smt_put_latency_seconds",
                "SMT put latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_put_bytes: register_histogram_vec_with_registry!(
                "smt_put_bytes",
                "SMT put size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            smt_remove_latency_seconds: register_histogram_vec_with_registry!(
                "smt_remove_latency_seconds",
                "SMT remove latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_remove_bytes: register_histogram_vec_with_registry!(
                "smt_remove_bytes",
                "SMT remove size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry
            )
            .unwrap(),
            smt_get_latency_seconds: register_histogram_vec_with_registry!(
                "smt_get_latency_seconds",
                "SMT get latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_get_bytes: register_histogram_vec_with_registry!(
                "smt_get_bytes",
                "SMT get returned data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            smt_get_with_proof_latency_seconds: register_histogram_vec_with_registry!(
                "smt_get_with_proof_latency_seconds",
                "SMT get with proof latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_get_with_proof_bytes: register_histogram_vec_with_registry!(
                "smt_get_with_proof_bytes",
                "SMT get with proof data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            smt_list_latency_seconds: register_histogram_vec_with_registry!(
                "smt_list_latency_seconds",
                "SMT list latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_list_bytes: register_histogram_vec_with_registry!(
                "smt_list_bytes",
                "SMT list data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            smt_iter_latency_seconds: register_histogram_vec_with_registry!(
                "smt_iter_latency_seconds",
                "SMT iter latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_puts_latency_seconds: register_histogram_vec_with_registry!(
                "smt_puts_latency_seconds",
                "SMT puts latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            smt_puts_bytes: register_histogram_vec_with_registry!(
                "smt_puts_bytes",
                "SMT puts data size in bytes",
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
