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

        // Check capacity and update tracking
        self.check_capacity_and_evict_if_needed(record_size)?;

        // Store the record
        self.store.kv_put(key, serialized)?;

        // Update tracking counters
        self.current_entries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.current_bytes.fetch_add(record_size, std::sync::atomic::Ordering::Relaxed);

        debug!(
            key = ?key,
            phase = ?record.phase,
            record_size,
            current_entries = self.current_entries.load(std::sync::atomic::Ordering::Relaxed),
            current_bytes = self.current_bytes.load(std::sync::atomic::Ordering::Relaxed),
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
            current_entries: self.current_entries.load(std::sync::atomic::Ordering::Relaxed),
            current_bytes: self.current_bytes.load(std::sync::atomic::Ordering::Relaxed),
            max_entries: self.max_entries,
            max_bytes: self.max_bytes,
        }
    }

    /// Check capacity and evict oldest records if needed (FIFO)
    fn check_capacity_and_evict_if_needed(&self, new_record_size: usize) -> Result<()> {
        let current_entries = self.current_entries.load(std::sync::atomic::Ordering::Relaxed);
        let current_bytes = self.current_bytes.load(std::sync::atomic::Ordering::Relaxed);

        // Check if adding this record would exceed capacity
        let would_exceed_entries = current_entries + 1 > self.max_entries;
        let would_exceed_bytes = current_bytes + new_record_size > self.max_bytes;

        if !would_exceed_entries && !would_exceed_bytes {
            return Ok(());
        }

        // Need to evict some records - implement FIFO eviction
        // For now, we'll just log a warning since full FIFO eviction requires iteration over keys
        // which is more complex to implement with the current store interface
        tracing::warn!(
            "Recycle bin capacity reached (entries={}, bytes={}). Consider increasing max_entries or max_bytes",
            current_entries,
            current_bytes
        );

        Ok(())
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
