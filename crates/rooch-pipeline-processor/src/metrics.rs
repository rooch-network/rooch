// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use prometheus::{
    register_histogram_vec_with_registry, register_int_counter_with_registry, HistogramVec,
    IntCounter, Registry,
};

#[derive(Debug)]
pub struct PipelineProcessorMetrics {
    pub pipeline_processor_execution_tx_latency_seconds: HistogramVec,
    pub pipeline_processor_execution_tx_bytes: HistogramVec,
    pub pipeline_processor_l1_block_gas_used: IntCounter,
    pub pipeline_processor_l1_tx_gas_used: IntCounter,
    pub pipeline_processor_l2_tx_gas_used: IntCounter,
}

impl PipelineProcessorMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        PipelineProcessorMetrics {
            pipeline_processor_execution_tx_latency_seconds: register_histogram_vec_with_registry!(
                "pipeline_processor_execution_tx_latency_seconds",
                "Pipeline processor execution tx latency in seconds",
                &["fn_name"],
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            pipeline_processor_execution_tx_bytes: register_histogram_vec_with_registry!(
                "pipeline_processor_execution_tx_bytes",
                "Pipeline processor execution tx size in bytes",
                &["fn_name"],
                prometheus::exponential_buckets(1.0, 4.0, 15)
                    .unwrap()
                    .to_vec(),
                registry,
            )
            .unwrap(),
            pipeline_processor_l1_block_gas_used: register_int_counter_with_registry!(
                "pipeline_processor_l1_block_gas_used",
                "Pipeline processor L1 block gas used total",
                registry,
            )
            .unwrap(),
            pipeline_processor_l1_tx_gas_used: register_int_counter_with_registry!(
                "pipeline_processor_l1_tx_gas_used",
                "Pipeline processor L1 tx gas used total",
                registry,
            )
            .unwrap(),
            pipeline_processor_l2_tx_gas_used: register_int_counter_with_registry!(
                "pipeline_processor_l2_tx_gas_used",
                "Pipeline processor L2 tx gas used total",
                registry,
            )
            .unwrap(),
        }
    }
}
