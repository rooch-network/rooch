// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_store::state_store::NodeRecycleDBStore;
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

// Import CodecKVStore trait for store methods
use raw_store::CodecKVStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecyclePhase {
    Incremental,
    SweepExpired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycleRecord {
    /// Original node bytes
    pub bytes: Vec<u8>,
    /// Which phase deleted it
    pub phase: RecyclePhase,
    /// Stale root for SweepExpired, cutoff_order for Incremental
    pub stale_root_or_cutoff: H256,
    /// Transaction order context (best effort)
    pub tx_order: u64,
    /// Deletion timestamp (seconds since epoch)
    pub deleted_at: u64,
    /// Optional note (e.g., refcount=0/missing)
    pub note: Option<String>,
}

pub struct RecycleBinStore {
    store: NodeRecycleDBStore,
    max_entries: usize,
    max_bytes: usize,
    current_entries: Arc<std::sync::atomic::AtomicUsize>,
    current_bytes: Arc<std::sync::atomic::AtomicUsize>,
}

impl RecycleBinStore {
    pub fn new(store: NodeRecycleDBStore, max_entries: usize, max_bytes: usize) -> Result<Self> {
        Ok(Self {
            store,
            max_entries,
            max_bytes,
            current_entries: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            current_bytes: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    pub fn put_record(&self, key: H256, record: RecycleRecord) -> Result<()> {
        let serialized = bcs::to_bytes(&record)?;
        let record_size = serialized.len();
        self.store.kv_put(key, serialized)?;

        debug!(
            key = ?key,
            phase = ?record.phase,
            record_size,
            "Stored record in recycle bin"
        );

        Ok(())
    }

    pub fn get_record(&self, key: &H256) -> Result<Option<RecycleRecord>> {
        match self.store.kv_get(*key)? {
            Some(data) => {
                let record = bcs::from_bytes(&data)?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    pub fn delete_record(&self, _key: &H256) -> Result<bool> {
        // For now, return false - we'll implement this later when we have proper access to raw_store traits
        Ok(false)
    }

    pub fn get_stats(&self) -> RecycleBinStats {
        RecycleBinStats {
            current_entries: 0,
            current_bytes: 0,
            max_entries: self.max_entries,
            max_bytes: self.max_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecycleBinStats {
    pub current_entries: usize,
    pub current_bytes: usize,
    pub max_entries: usize,
    pub max_bytes: usize,
}

impl std::fmt::Display for RecycleBinStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RecycleBin Statistics:")?;
        writeln!(
            f,
            "  Entries: {}/{} ({}%)",
            self.current_entries,
            self.max_entries,
            (self.current_entries * 100) / self.max_entries
        )?;
        writeln!(
            f,
            "  Bytes: {}/{} ({}%)",
            self.current_bytes,
            self.max_bytes,
            (self.current_bytes * 100) / self.max_bytes
        )?;
        Ok(())
    }
}
