// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    NODE_REFCOUNT_COLUMN_FAMILY_NAME, PRUNE_META_BLOOM_COLUMN_FAMILY_NAME,
    PRUNE_META_PHASE_COLUMN_FAMILY_NAME, REACH_SEEN_COLUMN_FAMILY_NAME,
    SMT_STALE_INDEX_COLUMN_FAMILY_NAME,
};
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_types::prune::PrunePhase;
use primitive_types::H256;
use raw_store::{derive_store, CodecKVStore, StoreInstance};

const META_KEY_PHASE: &str = "phase";
// const META_KEY_CURSOR: &str = "cursor"; // placeholder for future use
const META_KEY_BLOOM: &str = "bloom_snapshot";

derive_store!(
    ReachSeenDBStore,
    H256,
    Vec<u8>,
    REACH_SEEN_COLUMN_FAMILY_NAME
);
derive_store!(
    PruneMetaPhaseStore,
    String,
    PrunePhase,
    PRUNE_META_PHASE_COLUMN_FAMILY_NAME
);
derive_store!(
    PruneMetaBloomStore,
    String,
    BloomFilter,
    PRUNE_META_BLOOM_COLUMN_FAMILY_NAME
);
derive_store!(
    StaleIndexStore,
    (H256, H256),
    Vec<u8>,
    SMT_STALE_INDEX_COLUMN_FAMILY_NAME
);
derive_store!(
    NodeRefcountStore,
    H256,
    u32,
    NODE_REFCOUNT_COLUMN_FAMILY_NAME
);

pub trait PruneStore {
    fn load_prune_meta_phase(&self) -> Result<PrunePhase>;
    fn save_prune_meta_phase(&self, phase: PrunePhase) -> Result<()>;
    fn load_prune_meta_bloom(&self) -> Result<Option<BloomFilter>>;
    fn save_prune_meta_bloom(&self, phase: BloomFilter) -> Result<()>;
    fn list_before(&self, cutoff_root: H256, limit: usize) -> Result<Vec<(H256, H256)>>;
    fn inc_node_refcount(&self, key: H256) -> Result<()>;
    fn dec_node_refcount(&self, key: H256) -> Result<()>;
    fn remove_node_refcount(&self, key: H256) -> Result<()>;
    fn write_stale_indices(&self, stale: &[(H256, H256)]) -> Result<()>;

    fn get_stale_indice(&self, key: (H256, H256)) -> Result<Option<Vec<u8>>> ;
    fn remove_stale_indice(&self, key: (H256, H256)) -> Result<()>;
}

#[derive(Clone)]
pub struct PruneDBStore {
    pub reach_seen_store: ReachSeenDBStore,
    pub prune_meta_phase_store: PruneMetaPhaseStore,
    pub prune_meta_bloom_store: PruneMetaBloomStore,
    pub stale_index_store: StaleIndexStore,
    pub node_refcount_store: NodeRefcountStore,
}

impl PruneDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        PruneDBStore {
            reach_seen_store: ReachSeenDBStore::new(instance.clone()),
            prune_meta_phase_store: PruneMetaPhaseStore::new(instance.clone()),
            prune_meta_bloom_store: PruneMetaBloomStore::new(instance.clone()),
            stale_index_store: StaleIndexStore::new(instance.clone()),
            node_refcount_store: NodeRefcountStore::new(instance),
        }
    }

    pub fn load_prune_meta_phase(&self) -> Result<PrunePhase> {
        if let Some(phase) = self
            .prune_meta_phase_store
            .kv_get(META_KEY_PHASE.to_string())?
        {
            Ok(phase)
        } else {
            Ok(PrunePhase::BuildReach)
        }
    }

    pub fn save_prune_meta_phase(&self, phase: PrunePhase) -> Result<()> {
        self.prune_meta_phase_store
            .kv_put(META_KEY_PHASE.to_string(), phase)
    }

    pub fn load_prune_meta_bloom(&self) -> Result<Option<BloomFilter>> {
        self.prune_meta_bloom_store
            .kv_get(META_KEY_BLOOM.to_string())
    }

    pub fn save_prune_meta_bloom(&self, bloom: BloomFilter) -> Result<()> {
        self.prune_meta_bloom_store
            .kv_put(META_KEY_BLOOM.to_string(), bloom)
    }

    /// Fallback implementation: iterate CF and collect the first `limit` keys whose
    /// leading field (ts or root) is smaller than `cutoff_root`.
    /// This avoids the costly `self.keys()` (which calls StoreInstance::keys) and
    /// works without exposing RocksDB in upper layers.
    pub fn list_before(&self, cutoff_root: H256, limit: usize) -> Result<Vec<(H256, H256)>> {
        let mut out = Vec::with_capacity(limit);
        let mut iter = self.stale_index_store.iter()?;
        iter.seek_to_first();
        while let Some(item) = iter.next() {
            let (key, _): ((H256, H256), Vec<u8>) = item?;
            if key.0 < cutoff_root {
                out.push(key);
                if out.len() >= limit {
                    break;
                }
            }
        }
        Ok(out)
    }

    pub fn inc_node_refcount(&self, key: H256) -> Result<()> {
        let current = self.node_refcount_store.kv_get(key)?.unwrap_or(0);
        self.node_refcount_store.kv_put(key, current + 1)
    }

    pub fn dec_node_refcount(&self, key: H256) -> Result<()> {
        let current = self.node_refcount_store.kv_get(key)?.unwrap_or(1);
        let new = current.saturating_sub(1);
        if new == 0 {
            self.node_refcount_store.remove(key)
        } else {
            self.node_refcount_store.kv_put(key, new)
        }
    }

    /// Get current refcount, 0 if not present.
    pub fn get_node_refcount(&self, key: H256) -> Result<u32> {
        Ok(self.node_refcount_store.kv_get(key)?.unwrap_or(0))
    }

    pub fn remove_node_refcount(&self, key: H256) -> Result<()> {
        self.node_refcount_store.remove(key)
    }

    /// Write stale indices to cf_smt_stale (key = *timestamp-hash*, node_hash)
    /// and update refcount in one loop (non-atomic across CFs).
    pub fn write_stale_indices(&self, stale: &[(H256, H256)]) -> Result<()> {
        let ts = chrono::Utc::now().timestamp_millis() as u64;
        // Map 64-bit timestamp into H256 (low 64 bits store the value, upper bits are zero)
        let ts_h256 = H256::from_low_u64_be(ts);
        for (_root, node_hash) in stale {
            self.stale_index_store
                .kv_put((ts_h256, *node_hash), Vec::new())?;
            self.dec_node_refcount(*node_hash)?;
        }
        Ok(())
    }

    pub fn get_stale_indice(&self, key: (H256, H256)) -> Result<Option<Vec<u8>>> {
        self.stale_index_store.kv_get(key)
    }

    pub fn remove_stale_indice(&self, key: (H256, H256)) -> Result<()> {
        self.stale_index_store.remove(key)
    }
}
