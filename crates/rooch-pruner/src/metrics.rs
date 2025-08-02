// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct PrunerMetrics {
    pub pruner_reachable_nodes_scanned: HistogramVec,
    pub pruner_sweep_nodes_deleted: HistogramVec,
}

impl PrunerMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        PrunerMetrics {
            pruner_reachable_nodes_scanned: register_histogram_vec_with_registry!(
                "pruner_reachable_nodes_scanned",
                "Number of nodes scanned during reachable set build",
                &["phase"],
                prometheus::exponential_buckets(1.0, 2.0, 20)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),

            pruner_sweep_nodes_deleted: register_histogram_vec_with_registry!(
                "pruner_sweep_nodes_deleted",
                "Number of nodes deleted during sweep phase",
                &["phase"],
                prometheus::exponential_buckets(1.0, 2.0, 20)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
        }
    }
}
