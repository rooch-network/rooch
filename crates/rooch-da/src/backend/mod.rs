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
    async fn submit_batch(&self, batch: DABatch) -> anyhow::Result<()>;
}

// DABackendNopProxy is a no-op implementation of DABackendProxy
pub struct DABackendNopProxy;

#[async_trait]
impl DABackend for DABackendNopProxy {
    async fn submit_batch(&self, _batch: DABatch) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct DABackends {
    pub backends: Vec<Arc<dyn DABackend>>,
    pub backend_identifiers: Vec<String>,
    pub submit_threshold: usize,
    pub is_nop_backend: bool,
}

impl DABackends {
    const DEFAULT_SUBMIT_THRESHOLD: usize = 1;
    const DEFAULT_IS_NOP_BACKEND: bool = false;

    pub async fn initialize(
        config: Option<DABackendConfig>,
        genesis_namespace: String,
    ) -> anyhow::Result<Self> {
        let mut backends: Vec<Arc<dyn DABackend>> = Vec::new();
        let mut backend_names: Vec<String> = Vec::new();
        let mut submit_threshold = Self::DEFAULT_SUBMIT_THRESHOLD;
        let mut is_nop_backend = Self::DEFAULT_IS_NOP_BACKEND;

        let mut active_backends_count = 1; // Nop is always active
        if let Some(mut backend_config) = config {
            submit_threshold = backend_config.calculate_submit_threshold();
            active_backends_count = Self::load_backends_from_configs(
                &backend_config.backends,
                genesis_namespace,
                &mut backends,
                &mut backend_names,
            )
            .await?;
        } else {
            is_nop_backend = true;
            backends.push(Arc::new(DABackendNopProxy {}));
            backend_names.push("nop".to_string());
        }

        if active_backends_count < submit_threshold {
            return Err(anyhow!(
                "failed to start DA: not enough backends for future submissions. exp>= {} act: {}",
                submit_threshold,
                active_backends_count
            ));
        }

        Ok(Self {
            backends,
            backend_identifiers: backend_names,
            submit_threshold,
            is_nop_backend,
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
