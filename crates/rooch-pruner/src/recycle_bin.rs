// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use hex;
use moveos_store::state_store::NodeRecycleDBStore;
use moveos_types::h256::H256;
use raw_store::{CodecKVStore, CodecWriteBatch};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::Disks;
use tracing::{debug, error, warn};

pub const DEFAULT_LIST_LIMIT: usize = 100;

/// Result of disk space check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskSpaceStatus {
    /// Disk space is adequate
    Ok,
    /// Disk space is low, warning level
    Warning,
    /// Disk space is critical, emergency cleanup recommended
    Critical,
    /// Disk space is critically low, GC should stop
    Stop,
}

/// Paginated list result for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedListResult {
    pub entries: Vec<(H256, RecycleRecord)>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub page_size: usize,
}

#[derive(Debug, Clone)]
pub struct RecycleFilter {
    /// Delete entries older than this timestamp (seconds since epoch)
    pub older_than: Option<u64>,
    /// Delete entries newer than this timestamp (seconds since epoch)
    pub newer_than: Option<u64>,
    /// Delete entries with size >= min_size
    pub min_size: Option<usize>,
    /// Delete entries with size <= max_size
    pub max_size: Option<usize>,
}

impl RecycleFilter {
    /// Check if a record matches this filter
    pub fn matches(&self, record: &RecycleRecord) -> bool {
        // Check time range - use created_at instead of deleted_at
        if let Some(older_than) = self.older_than {
            if record.created_at >= older_than {
                return false;
            }
        }

        if let Some(newer_than) = self.newer_than {
            if record.created_at <= newer_than {
                return false;
            }
        }

        // Check size range
        if let Some(min_size) = self.min_size {
            if record.original_size < min_size {
                return false;
            }
        }

        if let Some(max_size) = self.max_size {
            if record.original_size > max_size {
                return false;
            }
        }

        true
    }
}

/// Strong backup configuration with immutable defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycleBinConfig {
    /// Strong backup mode - always true, no automatic deletion
    pub strong_backup: bool,
    /// Disk space warning threshold (percentage, default 20%)
    pub disk_space_warning_threshold: u64,
    /// Disk space critical threshold (percentage, default 10%) - trigger emergency cleanup
    pub disk_space_critical_threshold: u64,
    /// Disk space stop threshold (percentage, default 5%) - stop GC process
    pub disk_space_stop_threshold: u64,
    /// Enable disk space checks
    pub space_check_enabled: bool,
}

impl Default for RecycleBinConfig {
    fn default() -> Self {
        Self {
            strong_backup: true,               // Immutable default - never auto-delete
            disk_space_warning_threshold: 20,  // 20% - general warning
            disk_space_critical_threshold: 10, // 10% - trigger emergency cleanup
            disk_space_stop_threshold: 5,      // 5% - stop GC process
            space_check_enabled: true,
        }
    }
}

/// Simplified RecycleRecord structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycleRecord {
    /// Original node bytes - ESSENTIAL for recovery
    pub bytes: Vec<u8>,

    /// Creation timestamp - ESSENTIAL for lifecycle management and cleanup policies
    pub created_at: u64,

    /// Original node size - ESSENTIAL for storage management and statistics
    pub original_size: usize,
}

pub struct RecycleBinStore {
    store: NodeRecycleDBStore,
    config: RecycleBinConfig,
    // Database path obtained from store for disk space monitoring
    db_path: std::path::PathBuf,
    // Keep legacy fields for compatibility but don't use them for capacity enforcement
    current_entries: Arc<std::sync::atomic::AtomicUsize>,
    current_bytes: Arc<std::sync::atomic::AtomicUsize>,
}

impl RecycleBinStore {
    pub fn new(store: NodeRecycleDBStore) -> Result<Self> {
        Self::new_with_config(store, RecycleBinConfig::default())
    }

    pub fn new_with_config(store: NodeRecycleDBStore, config: RecycleBinConfig) -> Result<Self> {
        // Get database path from store
        let db_path = store
            .get_db_path()
            .ok_or_else(|| anyhow::anyhow!("Failed to get database path from store"))?;

        let instance = Self {
            store,
            config,
            db_path,
            current_entries: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            current_bytes: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };

        // Check disk space on initialization
        if instance.config.space_check_enabled {
            let status = instance.check_disk_space_status()?;
            if status == DiskSpaceStatus::Stop {
                let (total, available) = instance.get_disk_space_info()?;
                let available_percentage = (available as f64 / total as f64 * 100.0) as u64;
                return Err(anyhow::anyhow!(
                    "Cannot initialize RecycleBin: Disk space critically low ({}% available). Please free up disk space before starting GC.",
                    available_percentage
                ));
            }
        }

        Ok(instance)
    }

    pub fn put_record(&self, key: H256, record: RecycleRecord) -> Result<()> {
        let serialized = bcs::to_bytes(&record)?;
        let record_size = serialized.len();

        // Strong backup: No capacity checks or automatic eviction
        // Only perform disk space warnings if enabled
        if self.config.space_check_enabled {
            self.check_disk_space_and_warn()?;
        }

        // Store the record
        self.store.kv_put(key, serialized)?;

        // Update tracking counters
        self.current_entries
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.current_bytes
            .fetch_add(record_size, std::sync::atomic::Ordering::Relaxed);

        debug!(
            key = ?key,
            record_size,
            created_at = record.created_at,
            current_entries = self.current_entries.load(std::sync::atomic::Ordering::Relaxed),
            current_bytes = self.current_bytes.load(std::sync::atomic::Ordering::Relaxed),
            strong_backup = self.config.strong_backup,
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

    /// Create simplified record with essential data only
    pub fn create_record(&self, node_bytes: Vec<u8>) -> RecycleRecord {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        RecycleRecord {
            bytes: node_bytes.clone(),
            created_at: now,
            original_size: node_bytes.len(),
        }
    }

    /// Get disk space information for the database directory
    fn get_disk_space_info(&self) -> Result<(u64, u64)> {
        let disks = Disks::new_with_refreshed_list();
        if disks.is_empty() {
            return Err(anyhow::anyhow!("No disks found in the system"));
        }

        // Find the disk that contains our database directory
        for disk in &disks {
            if let Ok(canonical_db_path) = self.db_path.canonicalize() {
                // Check if the database directory is on this disk by comparing mount points
                if canonical_db_path.starts_with(disk.mount_point()) {
                    let total_space = disk.total_space();
                    let available_space = disk.available_space();
                    debug!(
                        "Monitoring disk space for database: {} (mount point: {}, total: {}GB, available: {}GB)",
                        self.db_path.display(),
                        disk.mount_point().display(),
                        total_space / (1024 * 1024 * 1024),
                        available_space / (1024 * 1024 * 1024)
                    );
                    return Ok((total_space, available_space));
                }
            }
        }

        // Fallback: use the first disk if we can't find the specific disk
        // This maintains backward compatibility
        warn!(
            "Could not find disk containing database: {}, falling back to first disk",
            self.db_path.display()
        );
        let disk = &disks[0];
        let total_space = disk.total_space();
        let available_space = disk.available_space();

        Ok((total_space, available_space))
    }

    /// Check disk space and return status
    fn check_disk_space_status(&self) -> Result<DiskSpaceStatus> {
        if !self.config.space_check_enabled {
            return Ok(DiskSpaceStatus::Ok);
        }

        let (total_space, available_space) = self.get_disk_space_info()?;

        if total_space == 0 {
            return Err(anyhow::anyhow!("Invalid disk space information"));
        }

        let available_percentage = (available_space as f64 / total_space as f64 * 100.0) as u64;

        if available_percentage <= self.config.disk_space_stop_threshold {
            Ok(DiskSpaceStatus::Stop)
        } else if available_percentage <= self.config.disk_space_critical_threshold {
            Ok(DiskSpaceStatus::Critical)
        } else if available_percentage <= self.config.disk_space_warning_threshold {
            Ok(DiskSpaceStatus::Warning)
        } else {
            Ok(DiskSpaceStatus::Ok)
        }
    }

    /// Check disk space and issue warnings or errors based on status
    fn check_disk_space_and_warn(&self) -> Result<()> {
        let status = self.check_disk_space_status()?;

        match status {
            DiskSpaceStatus::Ok => {
                // Disk space is adequate, no action needed
            }
            DiskSpaceStatus::Warning => {
                let (total, available) = self.get_disk_space_info()?;
                let available_percentage = (available as f64 / total as f64 * 100.0) as u64;
                warn!(
                    "WARNING: Disk space low ({}% available, {}GB total, {}GB available). Consider manual cleanup.",
                    available_percentage,
                    total / (1024 * 1024 * 1024),
                    available / (1024 * 1024 * 1024)
                );
            }
            DiskSpaceStatus::Critical => {
                let (total, available) = self.get_disk_space_info()?;
                let available_percentage = (available as f64 / total as f64 * 100.0) as u64;
                error!(
                    "CRITICAL: Disk space critically low ({}% available, {}GB total, {}GB available). Emergency cleanup recommended!",
                    available_percentage,
                    total / (1024 * 1024 * 1024),
                    available / (1024 * 1024 * 1024)
                );
            }
            DiskSpaceStatus::Stop => {
                let (total, available) = self.get_disk_space_info()?;
                let available_percentage = (available as f64 / total as f64 * 100.0) as u64;
                error!(
                    "STOP: Disk space exhausted ({}% available, {}GB total, {}GB available). GC process should stop to prevent system issues!",
                    available_percentage,
                    total / (1024 * 1024 * 1024),
                    available / (1024 * 1024 * 1024)
                );
                return Err(anyhow::anyhow!(
                    "Disk space critically low ({}% available). GC process stopped to prevent system damage.",
                    available_percentage
                ));
            }
        }

        Ok(())
    }

    pub fn delete_record(&self, key: &H256) -> Result<bool> {
        // Check if key exists first
        if let Some(record) = self.get_record(key)? {
            // Use CodecKVStore::remove to delete the record
            self.store.remove(*key)?;

            // Update statistics counters
            self.current_entries
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

            // Calculate the serialized record size for accurate byte counting
            let serialized_size = bcs::to_bytes(&record)?.len();
            self.current_bytes
                .fetch_sub(serialized_size, std::sync::atomic::Ordering::Relaxed);

            debug!(
                key = ?key,
                record_size = serialized_size,
                created_at = record.created_at,
                current_entries = self.current_entries.load(std::sync::atomic::Ordering::Relaxed),
                current_bytes = self.current_bytes.load(std::sync::atomic::Ordering::Relaxed),
                "Deleted record from recycle bin"
            );

            Ok(true)
        } else {
            debug!(key = ?key, "Record not found for deletion");
            Ok(false)
        }
    }

    /// List entries with keys for external use
    pub fn list_entries_with_keys(
        &self,
        filter: Option<RecycleFilter>,
        limit: Option<usize>,
    ) -> Result<Vec<(H256, RecycleRecord)>> {
        let mut results = Vec::new();

        // Get iterator
        let mut iter = self.store.iter()?;
        iter.seek_to_first();

        let max_count = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        for item in iter {
            if results.len() >= max_count {
                break;
            }

            let (key, value_bytes): (H256, Vec<u8>) = item?;

            // Deserialize record
            let record: RecycleRecord = bcs::from_bytes(&value_bytes)?;

            // Apply filter conditions
            if let Some(ref f) = filter {
                if !f.matches(&record) {
                    continue;
                }
            }

            results.push((key, record));
        }

        Ok(results)
    }

    /// List entries with cursor-based pagination
    pub fn list_entries_cursor(
        &self,
        filter: Option<RecycleFilter>,
        cursor: Option<H256>,
        page_size: usize,
    ) -> Result<PaginatedListResult> {
        let mut entries = Vec::new();
        let mut iter = self.store.iter()?;

        // Set starting position
        if let Some(cursor_key) = cursor {
            iter.seek(cursor_key.as_bytes().to_vec())?;
        } else {
            iter.seek_to_first();
        }

        // Skip the cursor key (since it was the last entry from previous page)
        if cursor.is_some() {
            iter.next();
        }

        // Read specified number of records
        let mut last_key: Option<H256> = None;
        let mut count = 0;
        let mut has_more = false;

        while count < page_size {
            match iter.next() {
                Some(item) => {
                    let (key, value_bytes): (H256, Vec<u8>) = item?;
                    let record: RecycleRecord = bcs::from_bytes(&value_bytes)?;

                    // Apply filter conditions
                    if let Some(ref f) = filter {
                        if !f.matches(&record) {
                            continue;
                        }
                    }

                    entries.push((key, record.clone()));
                    last_key = Some(key);
                    count += 1;
                }
                None => {
                    break;
                }
            }
        }

        // Check if there are more entries
        if count == page_size {
            has_more = iter.next().is_some();
        }

        Ok(PaginatedListResult {
            entries,
            next_cursor: last_key.map(|k| format!("0x{}", hex::encode(k.as_bytes()))),
            has_more,
            page_size,
        })
    }

    /// List all entries in the recycle bin with optional filtering
    pub fn list_entries(
        &self,
        filter: Option<RecycleFilter>,
        limit: Option<usize>,
    ) -> Result<Vec<RecycleRecord>> {
        let entries = self.list_entries_with_keys(filter, limit)?;
        Ok(entries.into_iter().map(|(_, record)| record).collect())
    }

    /// Delete entries from the recycle bin based on filter criteria
    pub fn delete_entries(&self, filter: &RecycleFilter, batch_size: usize) -> Result<usize> {
        let mut total_deleted = 0;

        loop {
            // Get a batch of matching entries
            let entries = self.list_entries_with_keys(Some(filter.clone()), Some(batch_size))?;

            if entries.is_empty() {
                break;
            }

            // Collect keys to delete
            let keys_to_delete: Vec<H256> = entries.iter().map(|(k, _)| *k).collect();

            // Create batch delete operation
            let batch = CodecWriteBatch::new_deletes(keys_to_delete.clone());
            self.store.write_batch(batch)?;

            // Update statistics
            let deleted_count = keys_to_delete.len();
            let deleted_bytes: usize = entries.iter().map(|(_, r)| r.original_size).sum();

            self.current_entries
                .fetch_sub(deleted_count, std::sync::atomic::Ordering::Relaxed);
            self.current_bytes
                .fetch_sub(deleted_bytes, std::sync::atomic::Ordering::Relaxed);

            total_deleted += deleted_count;

            debug!(
                deleted_count,
                deleted_bytes,
                total_deleted,
                current_entries = self
                    .current_entries
                    .load(std::sync::atomic::Ordering::Relaxed),
                current_bytes = self
                    .current_bytes
                    .load(std::sync::atomic::Ordering::Relaxed),
                "Batch deleted entries from recycle bin"
            );
        }

        Ok(total_deleted)
    }

    /// Delete entries older than the specified timestamp
    pub fn delete_entries_older_than(&self, cutoff_time: u64, batch_size: usize) -> Result<usize> {
        let filter = RecycleFilter {
            older_than: Some(cutoff_time),
            newer_than: None,
            min_size: None,
            max_size: None,
        };

        self.delete_entries(&filter, batch_size)
    }

    // preserve_recent_entries functionality removed - not implemented

    pub fn get_stats(&self) -> RecycleBinStats {
        let current_entries = self
            .current_entries
            .load(std::sync::atomic::Ordering::Relaxed);
        let current_bytes = self
            .current_bytes
            .load(std::sync::atomic::Ordering::Relaxed);

        RecycleBinStats {
            current_entries,
            current_bytes,
            max_entries: usize::MAX, // No limit with strong backup
            max_bytes: usize::MAX,   // No limit with strong backup
            strong_backup: self.config.strong_backup,
            space_warning_threshold: self.config.disk_space_warning_threshold,
            space_critical_threshold: self.config.disk_space_critical_threshold,
            space_stop_threshold: self.config.disk_space_stop_threshold,
        }
    }

    /// Get configuration for reference
    pub fn get_config(&self) -> &RecycleBinConfig {
        &self.config
    }

    // REMOVED: check_capacity_and_evict_if_needed method
    // Strong backup mode never automatically evicts records
    // All cleanup must be manual and explicit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycleBinStats {
    pub current_entries: usize,
    pub current_bytes: usize,
    pub max_entries: usize, // usize::MAX with strong backup (no limit)
    pub max_bytes: usize,   // usize::MAX with strong backup (no limit)
    pub strong_backup: bool,
    pub space_warning_threshold: u64,
    pub space_critical_threshold: u64,
    pub space_stop_threshold: u64,
}

impl std::fmt::Display for RecycleBinStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RecycleBin Statistics:")?;

        if self.strong_backup {
            writeln!(f, "  Strong Backup: ENABLED (no automatic deletion)")?;
        }

        let current_mb = self.current_bytes / (1024 * 1024);
        writeln!(
            f,
            "  Entries: {} (unlimited with strong backup)",
            self.current_entries
        )?;
        writeln!(
            f,
            "  Storage: {} MB (unlimited with strong backup)",
            current_mb
        )?;
        writeln!(
            f,
            "  Space Warning Threshold: {}%",
            self.space_warning_threshold
        )?;
        writeln!(
            f,
            "  Space Critical Threshold: {}%",
            self.space_critical_threshold
        )?;
        writeln!(f, "  Space Stop Threshold: {}%", self.space_stop_threshold)?;

        if self.strong_backup {
            writeln!(f, "  ⚠️  Manual cleanup required when disk space is low")?;
        }

        Ok(())
    }
}
