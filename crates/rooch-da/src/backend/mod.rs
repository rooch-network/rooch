// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::{AdapterSubmitStat, OpenDABackendManager};
use anyhow::anyhow;
use async_trait::async_trait;
use rooch_config::da_config::{DABackendConfig, DABackendConfigType};
use rooch_types::da::batch::DABatch;
use std::collections::HashMap;
use std::sync::Arc;

pub mod openda;

// manually set backend priority
// lower index means higher priority
pub const BACKENDS_PRIORITY: [&str; 5] = [
    "openda-fs",
    "openda-gcs",
    "openda-s3",
    "openda-avail",
    "openda-celestia",
];
pub const UNKNOWN_BACKEND_PRIORITY: usize = usize::MAX;

#[async_trait]
pub trait DABackend: Sync + Send {
    async fn submit_batch(&self, batch: Arc<DABatch>) -> anyhow::Result<()>;
    fn get_identifier(&self) -> String;
    fn get_adapter_stats(&self) -> AdapterSubmitStat;
}

pub struct DABackends {
    pub backends: Vec<Arc<dyn DABackend>>,
    pub submit_threshold: usize,
}

impl DABackends {
    /// Initializes the DA backends based on the given configuration and genesis namespace.
    pub async fn initialize(
        config: Option<DABackendConfig>,
        genesis_namespace: String,
    ) -> anyhow::Result<Self> {
        let mut backends = Vec::new();

        let submit_threshold = if let Some(mut backend_config) = config {
            let submit_threshold = backend_config.calculate_submit_threshold();

            // Load backends from the provided configuration
            let active_backends_count = Self::load_backends_from_configs(
                &backend_config.backends,
                genesis_namespace,
                &mut backends,
            )
            .await?;

            // Ensure enough backends are available for submission
            if active_backends_count < submit_threshold {
                return Err(anyhow!(
                    "failed to start DA: not enough backends for future submissions. exp >= {} act: {}",
                    submit_threshold,
                    active_backends_count
                ));
            }

            submit_threshold
        } else {
            0 // No configuration provided, default threshold is 0
        };

        let mut this = Self {
            backends,
            submit_threshold,
        };
        this.sort_backends();

        Ok(this)
    }

    // sort backends by their priority
    fn sort_backends(&mut self) {
        let priority_map: HashMap<&str, usize> = BACKENDS_PRIORITY
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        self.backends.sort_by(|a, b| {
            let a_priority = priority_map
                .get(a.get_identifier().as_str())
                .unwrap_or(&UNKNOWN_BACKEND_PRIORITY);
            let b_priority = priority_map
                .get(b.get_identifier().as_str())
                .unwrap_or(&UNKNOWN_BACKEND_PRIORITY);
            a_priority.cmp(b_priority)
        });
    }

    async fn load_backends_from_configs(
        backend_configs: &[DABackendConfigType],
        genesis_namespace: String,
        backends: &mut Vec<Arc<dyn DABackend>>,
    ) -> anyhow::Result<usize> {
        let mut available_backends = 0;
        for backend_type in backend_configs {
            #[allow(irrefutable_let_patterns)]
            if let DABackendConfigType::OpenDa(open_da_config) = backend_type {
                let mut open_da_config = open_da_config.clone();
                if open_da_config.namespace.is_none() {
                    open_da_config.namespace = Some(genesis_namespace.clone());
                }
                let backend = OpenDABackendManager::new(&open_da_config).await?;
                backends.push(Arc::new(backend));
                available_backends += 1;
            }
        }
        Ok(available_backends)
    }
}
