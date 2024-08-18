// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::metrics_util::LATENCY_SEC_BUCKETS;
use once_cell::sync::OnceCell;
use prometheus::{register_histogram_vec_with_registry, HistogramVec, Registry};
use std::sync::Arc;
use tap::TapFallible;
use tracing::warn;

#[derive(Debug)]
pub struct SQLiteMetrics {}

impl SQLiteMetrics {
    pub(crate) fn new(_registry: &Registry) -> Self {
        SQLiteMetrics {}
    }
}

#[derive(Debug)]
pub struct IndexerStoreMetrics {
    pub indexer_persist_or_update_or_delete_latency_seconds: HistogramVec,
    pub indexer_persist_or_update_or_delete_bytes: HistogramVec,
}

impl IndexerStoreMetrics {
    pub(crate) fn new(registry: &Registry) -> Self {
        IndexerStoreMetrics {
            indexer_persist_or_update_or_delete_latency_seconds:
                register_histogram_vec_with_registry!(
                    "indexer_persist_or_update_or_delete_latency_seconds",
                    "Indexer persist or update or delete latency in seconds",
                    &["fn_name"],
                    LATENCY_SEC_BUCKETS.to_vec(),
                    registry,
                )
                .unwrap(),
            indexer_persist_or_update_or_delete_bytes: register_histogram_vec_with_registry!(
                "indexer_persist_or_update_or_delete_bytes",
                "Indexer persist or update or delete in bytes",
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

#[derive(Debug)]
pub struct IndexerDBMetrics {
    pub indexer_store_metrics: IndexerStoreMetrics,
    pub sqlite_metrics: SQLiteMetrics,
}

static ONCE: OnceCell<Arc<IndexerDBMetrics>> = OnceCell::new();

impl IndexerDBMetrics {
    pub fn new(registry: &Registry) -> Self {
        IndexerDBMetrics {
            indexer_store_metrics: IndexerStoreMetrics::new(registry),
            sqlite_metrics: SQLiteMetrics::new(registry),
        }
    }

    pub fn init(registry: &Registry) -> &'static Arc<IndexerDBMetrics> {
        // Initialize this before creating any instance of StoreInstance
        // only ever initialize db metrics once with a registry whereas
        // in the code we might want to initialize it with different
        // registries.
        let _ = ONCE
            .set(Self::inner_init(registry))
            .tap_err(|_| warn!("IndexerDBMetrics registry overwritten"));
        ONCE.get().unwrap()
    }

    fn inner_init(registry: &Registry) -> Arc<IndexerDBMetrics> {
        Arc::new(IndexerDBMetrics::new(registry))
    }

    pub fn get() -> Option<&'static Arc<IndexerDBMetrics>> {
        ONCE.get()
    }

    pub fn get_or_init(registry: &Registry) -> &'static Arc<IndexerDBMetrics> {
        ONCE.get_or_init(|| Self::inner_init(registry).clone())
    }
}
