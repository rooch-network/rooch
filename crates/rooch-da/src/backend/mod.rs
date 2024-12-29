// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::OpenDABackend;
use anyhow::anyhow;
use async_trait::async_trait;
use rooch_config::da_config::{DABackendConfig, DABackendConfigType};
use rooch_types::da::batch::DABatch;
use std::sync::Arc;

pub mod openda;

#[async_trait]
pub trait DABackend: Sync + Send {
    async fn submit_batch(&self, batch: Arc<DABatch>) -> anyhow::Result<()>;
}

pub struct DABackends {
    pub backends: Vec<Arc<dyn DABackend>>,
    pub identifiers: Vec<String>,
    pub submit_threshold: usize,
}

impl DABackends {
    /// Initializes the DA backends based on the given configuration and genesis namespace.
    pub async fn initialize(
        config: Option<DABackendConfig>,
        genesis_namespace: String,
    ) -> anyhow::Result<Self> {
        let mut backends = Vec::new();
        let mut identifiers = Vec::new();

        let submit_threshold = if let Some(mut backend_config) = config {
            let submit_threshold = backend_config.calculate_submit_threshold();

            // Load backends from the provided configuration
            let active_backends_count = Self::load_backends_from_configs(
                &backend_config.backends,
                genesis_namespace,
                &mut backends,
                &mut identifiers,
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

        Ok(Self {
            backends,
            identifiers,
            submit_threshold,
        })
    }

    async fn load_backends_from_configs(
        backend_configs: &[DABackendConfigType],
        genesis_namespace: String,
        backends: &mut Vec<Arc<dyn DABackend>>,
        backend_names: &mut Vec<String>,
    ) -> anyhow::Result<usize> {
        let mut available_backends = 0;
        for backend_type in backend_configs {
            #[allow(irrefutable_let_patterns)]
            if let DABackendConfigType::OpenDa(openda_config) = backend_type {
                let backend = OpenDABackend::new(openda_config, genesis_namespace.clone()).await?;
                backends.push(Arc::new(backend));
                backend_names.push(format!("openda-{}", openda_config.scheme));
                available_backends += 1;
            }
        }
        Ok(available_backends)
    }
}
