// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use prometheus::{
    register_counter_vec_with_registry, register_gauge_vec_with_registry,
    register_histogram_vec_with_registry, CounterVec, GaugeVec, HistogramVec, Registry,
};

#[derive(Debug)]
pub struct PrunerMetrics {
    pub pruner_reachable_nodes_scanned: HistogramVec,
    pub pruner_sweep_nodes_deleted: HistogramVec,
    pub pruner_bloom_filter_size_bytes: GaugeVec,
    pub pruner_current_phase: GaugeVec,
    pub pruner_processing_speed_nodes_per_sec: HistogramVec,
    pub pruner_error_count: CounterVec,
    pub pruner_disk_space_reclaimed_bytes: CounterVec,
}

impl PrunerMetrics {
    pub fn new(registry: &Registry) -> Self {
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

            pruner_bloom_filter_size_bytes: register_gauge_vec_with_registry!(
                "pruner_bloom_filter_size_bytes",
                "Bloom filter size in bytes",
                &["phase"],
                registry,
            )
            .unwrap(),

            pruner_current_phase: register_gauge_vec_with_registry!(
                "pruner_current_phase",
                "Current pruning phase (0=BuildReach, 1=SweepExpired, 2=Incremental)",
                &["phase"],
                registry,
            )
            .unwrap(),

            pruner_processing_speed_nodes_per_sec: register_histogram_vec_with_registry!(
                "pruner_processing_speed_nodes_per_sec",
                "Processing speed in nodes per second",
                &["phase"],
                prometheus::exponential_buckets(1.0, 2.0, 20)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),

            pruner_error_count: register_counter_vec_with_registry!(
                "pruner_error_count",
                "Number of errors encountered during pruning",
                &["operation", "phase"],
                registry,
            )
            .unwrap(),

            pruner_disk_space_reclaimed_bytes: register_counter_vec_with_registry!(
                "pruner_disk_space_reclaimed_bytes",
                "Estimated disk space reclaimed in bytes",
                &["phase"],
                registry,
            )
            .unwrap(),
        }
    }
}
