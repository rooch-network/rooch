use crate::config_store::ConfigDBStore;
use crate::state_store::incremental_sweep::IncrementalSweep;
use crate::state_store::metrics::StateDBMetrics;
use crate::state_store::{reachability::ReachableBuilder, sweep_expired::SweepExpired};
use crate::state_store::{NodeDBStore, PruneMetaStore, ReachSeenDBStore};
use crate::state_store::{NodeRefcountStore, StaleIndexStore};
use anyhow::Result;
use parking_lot::Mutex;
use primitive_types::H256;
use raw_store::CodecKVStore;
use raw_store::SchemaStore;
use rooch_common::bloom::BloomFilter;
use rooch_config::PruneConfig;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrunePhase {
    BuildReach,
    SweepExpired,
    Incremental,
}

const META_KEY_PHASE: &str = "phase";
const META_KEY_CURSOR: &str = "cursor"; // placeholder for future use
const META_KEY_BLOOM: &str = "bloom_snapshot";

fn load_phase(meta: &PruneMetaStore) -> Result<PrunePhase> {
    if let Some(bytes) = meta.kv_get(META_KEY_PHASE.to_string())? {
        Ok(bcs::from_bytes::<PrunePhase>(&bytes)?)
    } else {
        Ok(PrunePhase::BuildReach)
    }
}

fn save_phase(meta: &PruneMetaStore, phase: PrunePhase) -> Result<()> {
    meta.kv_put(META_KEY_PHASE.to_string(), bcs::to_bytes(&phase)?)
}

pub struct StateDBPruner {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl StateDBPruner {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        cfg: Arc<PruneConfig>,
        node_store: Arc<NodeDBStore>,
        reach_seen: Option<Arc<ReachSeenDBStore>>,
        meta_store: Arc<PruneMetaStore>,
        metrics: Arc<StateDBMetrics>,
    ) -> Result<Self> {
        if !cfg.enable {
            return Ok(Self {
                handle: None,
                running: Arc::new(AtomicBool::new(false)),
            });
        }

        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let handle = thread::spawn(move || {
            // Load bloom snapshot if exists
            let bloom = if let Ok(Some(bytes)) = meta_store.kv_get(META_KEY_BLOOM.to_string()) {
                match BloomFilter::from_bytes(&bytes, 4) {
                    Ok(bf) => Arc::new(Mutex::new(bf)),
                    Err(_) => Arc::new(Mutex::new(BloomFilter::new(cfg.bloom_bits as usize, 4))),
                }
            } else {
                Arc::new(Mutex::new(BloomFilter::new(cfg.bloom_bits as usize, 4)))
            };

            loop {
                if !thread_running.load(Ordering::Relaxed) {
                    break;
                }

                // load current phase
                let phase = load_phase(&meta_store).unwrap_or(PrunePhase::BuildReach);

                match phase {
                    PrunePhase::BuildReach => {
                        // Determine current live root via StartupInfo
                        let config_store =
                            ConfigDBStore::new(node_store.as_ref().get_store().store().clone());
                        let live_roots = config_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| vec![info.state_root]))
                            .unwrap_or_default();
                        let builder = ReachableBuilder::new(
                            node_store.clone(),
                            reach_seen.clone(),
                            bloom.clone(),
                            metrics.clone(),
                        );
                        let _ = builder.build(live_roots, num_cpus::get());
                        // Persist bloom snapshot after reachability phase
                        {
                            let bytes = bloom.lock().to_bytes();
                            let _ = meta_store.kv_put(META_KEY_BLOOM.to_string(), bytes);
                        }
                        save_phase(&meta_store, PrunePhase::SweepExpired).ok();
                    }
                    PrunePhase::SweepExpired => {
                        let stale_store =
                            StaleIndexStore::new(node_store.as_ref().get_store().store().clone());
                        // Use cutoff_root = current live root for first sweep
                        let config_store =
                            ConfigDBStore::new(node_store.as_ref().get_store().store().clone());
                        let cutoff_root = config_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| info.state_root))
                            .unwrap_or_else(H256::zero);
                        let expired_pairs = stale_store
                            .list_before(cutoff_root, cfg.scan_batch)
                            .unwrap_or_default();
                        let mut expired_roots: Vec<H256> =
                            expired_pairs.into_iter().map(|(root, _)| root).collect();
                        expired_roots.sort();
                        expired_roots.dedup();
                        let sweeper = SweepExpired::new(
                            node_store.clone(),
                            reach_seen.clone(),
                            bloom.clone(),
                            metrics.clone(),
                        );
                        let _ = sweeper.sweep(expired_roots, num_cpus::get());
                        // Persist bloom snapshot after sweep phase (in case items added)
                        {
                            let bytes = bloom.lock().to_bytes();
                            let _ = meta_store.kv_put(META_KEY_BLOOM.to_string(), bytes);
                        }
                        // after sweep finished once, switch to incremental phase
                        save_phase(&meta_store, PrunePhase::Incremental).ok();
                    }
                    PrunePhase::Incremental => {
                        let stale_store =
                            StaleIndexStore::new(node_store.as_ref().get_store().store().clone());
                        let ref_store =
                            NodeRefcountStore::new(node_store.as_ref().get_store().store().clone());
                        // Cutoff root placeholder: in a real system derive from time window.
                        let cutoff_root = H256::zero();
                        let sweeper = IncrementalSweep::new(
                            node_store.clone(),
                            Arc::new(stale_store),
                            Arc::new(ref_store),
                        );
                        let _ = sweeper.sweep(cutoff_root, cfg.scan_batch as usize);
                    }
                }

                thread::sleep(Duration::from_secs(cfg.interval_s as u64));
            }
        });

        Ok(Self {
            handle: Some(handle),
            running,
        })
    }

    pub fn stop(self) {
        if let Some(h) = self.handle {
            self.running.store(false, Ordering::Relaxed);
            let _ = h.join();
        }
    }
}
