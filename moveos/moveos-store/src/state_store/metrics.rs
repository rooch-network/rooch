// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, register_gauge_vec_with_registry, register_counter_vec_with_registry, HistogramVec, GaugeVec, CounterVec, Registry};

#[derive(Debug)]
pub struct StateDBMetrics {
    pub state_update_fields_latency_seconds: HistogramVec,
    pub state_update_fields_bytes: HistogramVec,
    pub state_update_nodes_latency_seconds: HistogramVec,
    pub state_update_nodes_bytes: HistogramVec,
    pub state_change_set_to_nodes_latency_seconds: HistogramVec,
    pub state_change_set_to_nodes_bytes: HistogramVec,
    pub state_iter_latency_seconds: HistogramVec,
    pub state_get_field_at_latency_seconds: HistogramVec,
    pub state_get_field_at_bytes: HistogramVec,
    pub state_list_fields_at_latency_seconds: HistogramVec,
    pub state_list_fields_at_bytes: HistogramVec,
    pub pruner_reachable_nodes_scanned: HistogramVec,
    pub pruner_sweep_nodes_deleted: HistogramVec,
    // Additional pruner metrics for better monitoring
    pub pruner_disk_space_reclaimed_bytes: CounterVec,
    pub pruner_current_phase: GaugeVec,
    pub pruner_bloom_filter_size_bytes: GaugeVec,
    pub pruner_processing_speed_nodes_per_sec: HistogramVec,
    pub pruner_error_count: CounterVec,
}

impl StateDBMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        StateDBMetrics {
            state_update_fields_latency_seconds: register_histogram_vec_with_registry!(
                "state_update_fields_latency_seconds",
                "State update fields latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_update_fields_bytes: register_histogram_vec_with_registry!(
                "state_update_fields_bytes",
                "State update fields data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            state_update_nodes_latency_seconds: register_histogram_vec_with_registry!(
                "state_update_nodes_latency_seconds",
                "State update nodes latency in seconds",
                &["fn_name"],
                registry,
            )
            .unwrap(),
            state_update_nodes_bytes: register_histogram_vec_with_registry!(
                "state_update_nodes_bytes",
                "State update nodes data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry
            )
            .unwrap(),
            state_change_set_to_nodes_latency_seconds: register_histogram_vec_with_registry!(
                "state_apply_change_set_latency_seconds",
                "State apply change set latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_change_set_to_nodes_bytes: register_histogram_vec_with_registry!(
                "state_apply_change_set_bytes",
                "State apply change set data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            state_iter_latency_seconds: register_histogram_vec_with_registry!(
                "state_iter_latency_seconds",
                "State iter latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_get_field_at_latency_seconds: register_histogram_vec_with_registry!(
                "state_get_field_at_latency_seconds",
                "State get field latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_get_field_at_bytes: register_histogram_vec_with_registry!(
                "state_get_field_at_bytes",
                "State get field data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            state_list_fields_at_latency_seconds: register_histogram_vec_with_registry!(
                "state_list_fields_at_latency_seconds",
                "State list fields latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_list_fields_at_bytes: register_histogram_vec_with_registry!(
                "state_list_fields_at_bytes",
                "State list fields data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
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

            // Additional pruner metrics
            pruner_disk_space_reclaimed_bytes: register_counter_vec_with_registry!(
                "pruner_disk_space_reclaimed_bytes_total",
                "Total disk space reclaimed by pruner in bytes",
                &["phase"],
                registry,
            )
            .unwrap(),

            pruner_current_phase: register_gauge_vec_with_registry!(
                "pruner_current_phase",
                "Current pruning phase (0=BuildReach, 1=SweepExpired, 2=Incremental)",
                &["phase_name"],
                registry,
            )
            .unwrap(),

            pruner_bloom_filter_size_bytes: register_gauge_vec_with_registry!(
                "pruner_bloom_filter_size_bytes",
                "Current bloom filter size in bytes",
                &[],
                registry,
            )
            .unwrap(),

            pruner_processing_speed_nodes_per_sec: register_histogram_vec_with_registry!(
                "pruner_processing_speed_nodes_per_sec",
                "Processing speed in nodes per second",
                &["operation"],
                prometheus::exponential_buckets(1.0, 2.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),

            pruner_error_count: register_counter_vec_with_registry!(
                "pruner_error_count_total",
                "Total number of pruner errors",
                &["error_type", "phase"],
                registry,
            )
            .unwrap(),
        }
    }

    pub fn init(registry: &Registry) {
        // This creates and immediately drops the metrics to register them
        // The actual metrics will be accessed through the store instances
        let _ = Self::new(registry);
    }
}
