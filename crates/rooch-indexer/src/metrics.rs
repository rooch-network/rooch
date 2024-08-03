// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct IndexerReaderMetrics {
    pub indexer_reader_query_latency_seconds: HistogramVec,
}

impl IndexerReaderMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        IndexerReaderMetrics {
            indexer_reader_query_latency_seconds: register_histogram_vec_with_registry!(
                "indexer_reader_query_latency_seconds",
                "Indexer reader query latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
        }
    }
}
