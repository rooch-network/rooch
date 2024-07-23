// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};

#[derive(Debug)]
pub struct StateDBMetrics {
    pub state_update_fields_latency: HistogramVec,
    pub state_update_fields_bytes: HistogramVec,
    pub state_update_nodes_latency: HistogramVec,
    pub state_update_nodes_bytes: HistogramVec,
    pub state_apply_change_set_latency: HistogramVec,
    pub state_apply_change_set_bytes: HistogramVec,
    pub state_iter_latency: HistogramVec,
    pub state_get_field_at_latency: HistogramVec,
    pub state_get_field_at_bytes: HistogramVec,
    pub state_list_fields_at_latency: HistogramVec,
    pub state_list_fields_at_bytes: HistogramVec,
}

impl StateDBMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        StateDBMetrics {
            state_update_fields_latency: register_histogram_vec_with_registry!(
                "state_update_fields_latency",
                "State update fields latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_update_fields_bytes: register_histogram_vec_with_registry!(
                "state_update_fields_bytes",
                "State update fileds data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            state_update_nodes_latency: register_histogram_vec_with_registry!(
                "state_update_nodes_latency",
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
            state_apply_change_set_latency: register_histogram_vec_with_registry!(
                "state_apply_change_set_latency",
                "State apply change set latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_apply_change_set_bytes: register_histogram_vec_with_registry!(
                "state_apply_change_set_bytes",
                "State apply change set data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            state_iter_latency: register_histogram_vec_with_registry!(
                "state_iter_latency",
                "State iter latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_get_field_at_latency: register_histogram_vec_with_registry!(
                "state_get_field_at_latency",
                "State get field latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            state_get_field_at_bytes: register_histogram_vec_with_registry!(
                "state_get_field_at_bytes",
                "State get filed data size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            state_list_fields_at_latency: register_histogram_vec_with_registry!(
                "state_list_fields_at_latency",
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
        }
    }
}
