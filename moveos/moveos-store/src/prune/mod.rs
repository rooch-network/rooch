// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    NODE_REFCOUNT_COLUMN_FAMILY_NAME, PRUNE_META_BLOOM_COLUMN_FAMILY_NAME,
    PRUNE_META_DELETED_ROOTS_BLOOM_COLUMN_FAMILY_NAME, PRUNE_META_PHASE_COLUMN_FAMILY_NAME,
    PRUNE_META_SNAPSHOT_COLUMN_FAMILY_NAME, REACH_SEEN_COLUMN_FAMILY_NAME,
    SMT_STALE_INDEX_COLUMN_FAMILY_NAME,
};
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_types::prune::{PrunePhase, PruneSnapshot};
use primitive_types::H256;
use raw_store::{derive_store, CodecKVStore, StoreInstance};
use tracing::warn;

const META_KEY_PHASE: &str = "phase";
// const META_KEY_CURSOR: &str = "cursor"; // placeholder for future use
const META_KEY_BLOOM: &str = "bloom_snapshot";
const META_KEY_SNAPSHOT: &str = "meta_snapshot";
const META_KEY_DELETED_STATE_ROOT_BLOOM: &str = "deleted_state_root_bloom";

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
derive_store!(
    PruneMetaSnapshotStore,
    String,
    PruneSnapshot,
    PRUNE_META_SNAPSHOT_COLUMN_FAMILY_NAME
);
derive_store!(
    DeletedStateRootBloomStore,
    String,
    BloomFilter,
    PRUNE_META_DELETED_ROOTS_BLOOM_COLUMN_FAMILY_NAME
);

pub trait PruneStore {
    fn load_prune_meta_phase(&self) -> Result<PrunePhase>;
    fn save_prune_meta_phase(&self, phase: PrunePhase) -> Result<()>;
    fn load_prune_meta_bloom(&self) -> Result<Option<BloomFilter>>;
    fn save_prune_meta_bloom(&self, phase: BloomFilter) -> Result<()>;
    /// List stale indices whose tx_order (stored in key.0) is earlier than `cutoff_order`.
    fn list_before(&self, cutoff_order: u64, limit: usize) -> Result<Vec<(H256, H256)>>;
    fn inc_node_refcount(&self, key: H256) -> Result<()>;
    fn dec_node_refcount(&self, key: H256) -> Result<()>;
    fn remove_node_refcount(&self, key: H256) -> Result<()>;
    fn write_stale_indices(&self, tx_order: u64, stale: &[(H256, H256)]) -> Result<()>;

    fn get_stale_indice(&self, key: (H256, H256)) -> Result<Option<Vec<u8>>>;
    fn remove_stale_indice(&self, key: (H256, H256)) -> Result<()>;

    fn save_prune_meta_snapshot(&self, snap: PruneSnapshot) -> Result<()>;

    fn load_prune_meta_snapshot(&self) -> Result<Option<PruneSnapshot>>;

    // New methods for tracking deleted state roots
    fn load_deleted_state_root_bloom(&self) -> Result<Option<BloomFilter>>;
    fn save_deleted_state_root_bloom(&self, bloom: BloomFilter) -> Result<()>;
}

#[derive(Clone)]
pub struct PruneDBStore {
    pub reach_seen_store: ReachSeenDBStore,
    pub prune_meta_phase_store: PruneMetaPhaseStore,
    pub prune_meta_bloom_store: PruneMetaBloomStore,
    pub prune_meta_snapshot_store: PruneMetaSnapshotStore,
    pub deleted_state_root_bloom_store: DeletedStateRootBloomStore,
    pub stale_index_store: StaleIndexStore,
    pub node_refcount_store: NodeRefcountStore,
}

impl PruneDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        PruneDBStore {
            reach_seen_store: ReachSeenDBStore::new(instance.clone()),
            prune_meta_phase_store: PruneMetaPhaseStore::new(instance.clone()),
            prune_meta_bloom_store: PruneMetaBloomStore::new(instance.clone()),
            prune_meta_snapshot_store: PruneMetaSnapshotStore::new(instance.clone()),
            deleted_state_root_bloom_store: DeletedStateRootBloomStore::new(instance.clone()),
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

    /// Persist snapshot captured in BuildReach.
    pub fn save_prune_meta_snapshot(&self, snap: PruneSnapshot) -> Result<()> {
        self.prune_meta_snapshot_store
            .kv_put(META_KEY_SNAPSHOT.to_string(), snap)
    }

    /// Load snapshot for SweepExpired.
    pub fn load_prune_meta_snapshot(&self) -> Result<Option<PruneSnapshot>> {
        self.prune_meta_snapshot_store
            .kv_get(META_KEY_SNAPSHOT.to_string())
    }

    /// Load BloomFilter of deleted state roots.
    pub fn load_deleted_state_root_bloom(&self) -> Result<Option<BloomFilter>> {
        self.deleted_state_root_bloom_store
            .kv_get(META_KEY_DELETED_STATE_ROOT_BLOOM.to_string())
    }

    /// Save BloomFilter of deleted state roots.
    pub fn save_deleted_state_root_bloom(&self, bloom: BloomFilter) -> Result<()> {
        self.deleted_state_root_bloom_store
            .kv_put(META_KEY_DELETED_STATE_ROOT_BLOOM.to_string(), bloom)
    }

    /// Fallback implementation: iterate CF and collect the first `limit` keys whose
    /// leading field (tx_order) is smaller than `cutoff_order`.
    /// This avoids the costly `self.keys()` (which calls StoreInstance::keys) and
    /// works without exposing RocksDB in upper layers.
    pub fn list_before(&self, cutoff_order: u64, limit: usize) -> Result<Vec<(H256, H256)>> {
        let cutoff = H256::from_low_u64_be(cutoff_order);
        let mut out = Vec::with_capacity(limit);
        let mut iter = self.stale_index_store.iter()?;
        iter.seek_to_first();
        for item in iter {
            let (key, _): ((H256, H256), Vec<u8>) = item?;
            if key.0 < cutoff {
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
        match self.node_refcount_store.kv_get(key)? {
            Some(current) => {
                let new = current.saturating_sub(1);
                if new == 0 {
                    self.node_refcount_store.remove(key)
                } else {
                    self.node_refcount_store.kv_put(key, new)
                }
            }
            None => {
                warn!(
                    ?key,
                    "dec_node_refcount called for missing refcount entry, skipping"
                );
                Ok(())
            }
        }
    }

    /// Get current refcount, 0 if not present.
    /// Return Some(refcount) if present, None if missing.
    pub fn get_node_refcount(&self, key: H256) -> Result<Option<u32>> {
        self.node_refcount_store.kv_get(key)
    }

    pub fn remove_node_refcount(&self, key: H256) -> Result<()> {
        self.node_refcount_store.remove(key)
    }

    /// Write stale indices to cf_smt_stale (key = *tx_order-hash*, node_hash)
    /// and update refcount in one loop (non-atomic across CFs).
    pub fn write_stale_indices(&self, tx_order: u64, stale: &[(H256, H256)]) -> Result<()> {
        // Map 64-bit tx_order into H256 (low 64 bits store the value, upper bits are zero)
        let order_h256 = H256::from_low_u64_be(tx_order);
        for (_root, node_hash) in stale {
            self.stale_index_store
                .kv_put((order_h256, *node_hash), Vec::new())?;
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
