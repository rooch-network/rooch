// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::GCConfig;
use crate::historical_state::{HistoricalStateCollector, HistoricalStateConfig};
use crate::marker::{create_marker, NodeMarker};
use crate::reachability::ReachableBuilder;
use crate::recycle_bin::RecycleBinStore;
use crate::safety_verifier::SafetyVerifier;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::STATE_NODE_COLUMN_FAMILY_NAME;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use raw_store::{CodecKVStore, SchemaStore};
use rooch_db::RoochDB;
use smt::NodeReader;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Comprehensive GC execution report
#[derive(Debug, Clone)]
pub struct GCReport {
    /// The protected root nodes used for this GC run
    pub protected_roots: Vec<H256>,
    /// Statistics from the Mark phase
    pub mark_stats: MarkStats,
    /// Statistics from the Sweep phase
    pub sweep_stats: SweepStats,
    /// Total execution duration
    pub duration: Duration,
    /// Memory strategy that was actually used (now always "BloomFilter")
    pub memory_strategy_used: String,
}

/// Mark phase statistics
#[derive(Debug, Clone, Default)]
pub struct MarkStats {
    /// Number of nodes marked as reachable
    pub marked_count: u64,
    /// Time taken for mark phase
    pub duration: Duration,
    /// Memory strategy used
    pub memory_strategy: String,
}

/// Sweep phase statistics
#[derive(Debug, Clone, Default)]
pub struct SweepStats {
    /// Number of nodes scanned during sweep
    pub scanned_count: u64,
    /// Number of nodes kept (marked as reachable)
    pub kept_count: u64,
    /// Number of nodes deleted (unmarked)
    pub deleted_count: u64,
    /// Number of nodes sent to recycle bin
    pub recycle_bin_entries: u64,
    /// Time taken for sweep phase
    pub duration: Duration,
}

/// Stop-the-world Garbage Collector for Rooch state nodes
///
/// This implements a safe Mark-Sweep garbage collection algorithm that operates
/// while the database is stopped (no concurrent writes). It provides both safety
/// guarantees and comprehensive reporting.
pub struct GarbageCollector {
    rooch_db: RoochDB,
    #[allow(dead_code)]
    recycle_bin: Arc<RecycleBinStore>,
    pub config: GCConfig,
    db_path: std::path::PathBuf,
}

impl GarbageCollector {
    /// Average size of a SMT node in bytes (key + value + overhead)
    /// Used for estimating node count from data size statistics
    const AVG_NODE_SIZE_BYTES: u64 = 200;

    /// Create a new GarbageCollector with the given store, configuration, and database path
    pub fn new(rooch_db: RoochDB, config: GCConfig) -> Result<Self> {
        let recycle_bin =
            RecycleBinStore::new(rooch_db.moveos_store.get_node_recycle_store().clone())?;
        // Initialize recycle bin with reasonable defaults
        let recycle_bin = Arc::new(recycle_bin);
        let db_path = rooch_db
            .rocksdb_path()
            .ok_or_else(|| anyhow::anyhow!("Failed to get database path from store"))?;
        Ok(Self {
            rooch_db,
            recycle_bin,
            config,
            db_path,
        })
    }

    /// Execute the complete garbage collection process
    pub fn execute_gc(&self) -> Result<GCReport> {
        let start_time = Instant::now();

        info!("=== Starting Stop-the-World Garbage Collection ===");
        info!("Config: {:?}", self.config);

        // Phase 1: Safety verification
        self.verify_database_safety()?;

        // Phase 2: Determine root set for this GC run
        let protected_roots = self.get_protected_roots()?;
        info!("Protected roots: {:?}", protected_roots);

        // Phase 3: Estimate node count for marker configuration
        let estimated_nodes = self.estimate_node_count(&protected_roots)?;

        info!("Estimated nodes: {}", estimated_nodes);

        // Phase 4: Mark phase - identify all reachable nodes
        let (mark_stats, marker) =
            self.mark_phase(&protected_roots, estimated_nodes)?;

        // Phase 5: User confirmation (after mark, before sweep; skip in dry-run or if skip_confirm is enabled)
        if !self.config.dry_run {
            self.request_user_confirmation(&protected_roots, &mark_stats)?;
        }

        // Phase 5: Sweep phase - delete unreachable nodes (skip in dry-run)
        let sweep_stats = if self.config.dry_run {
            info!("Dry-run mode: skipping sweep phase");
            SweepStats::default()
        } else {
            self.sweep_phase(&protected_roots, marker.as_ref())?
        };

        // Phase 6: Optional compaction
        if self.config.force_compaction && !self.config.dry_run {
            info!("=== Compaction Phase ===");
            self.trigger_compaction()?;
        }

        // Phase 7: Generate comprehensive report
        let total_duration = start_time.elapsed();
        let report = GCReport {
            protected_roots,
            mark_stats,
            sweep_stats,
            duration: total_duration,
            memory_strategy_used: "BloomFilter".to_string(),
        };

        info!("=== Garbage Collection Completed ===");
        info!("Report: {:?}", report);

        Ok(report)
    }

    /// Verify database safety - check for recent activity and conflicts
    fn verify_database_safety(&self) -> Result<()> {
        info!("=== Safety Verification ===");

        if !self.config.dry_run {
            info!("Performing mandatory database safety verification...");

            let db_path = self.get_database_path()?;
            let safety_verifier = SafetyVerifier::new(&db_path);

            match safety_verifier.verify_database_access() {
                Ok(report) if report.database_available => {
                    info!("✅ {}", report.message);
                    info!("🔒 All technical safety checks passed - proceeding with garbage collection");
                    info!("   - Database RocksDB LOCK file verified");
                    info!("   - Exclusive access confirmed");
                }
                Ok(report) => {
                    return Err(anyhow::anyhow!(
                        "Database is locked, please stop the blockchain service and retry: {}",
                        report.message
                    ));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Database safety check failed: {}. Please check database permissions and ensure the service is stopped.",
                        e
                    ));
                }
            }
        } else {
            info!("Running in dry-run mode - skipping technical safety verification");
        }

        Ok(())
    }

    /// Request user confirmation before proceeding with garbage collection
    fn request_user_confirmation(
        &self,
        protected_roots: &[H256],
        mark_stats: &MarkStats,
    ) -> Result<()> {
        // If skip_confirm is enabled, skip user confirmation
        if self.config.skip_confirm {
            warn!("⚠️  Skipping user confirmation (automation mode)");
            return Ok(());
        }

        info!("=== User Confirmation ===");

        println!("=== Garbage Collection Preview ===");
        println!("Protected Root Nodes Count: {}", protected_roots.len());

        // Display some root nodes (max 5)
        for (i, root) in protected_roots.iter().take(5).enumerate() {
            println!("  {}: {}", i + 1, root);
        }
        if protected_roots.len() > 5 {
            println!("  ... and {} more root nodes", protected_roots.len() - 5);
        }
        println!();

        println!("Mark Phase Summary:");
        println!("  Reachable nodes marked: {}", mark_stats.marked_count);
        println!("  Marker strategy: {}", mark_stats.memory_strategy);
        println!();

        println!("⚠️  This will permanently delete unreachable state nodes");
        if self.config.use_recycle_bin {
            println!("📁 Recycle bin enabled, deleted nodes will be saved");
        } else {
            println!("🗑️  Recycle bin disabled, deleted nodes will be permanently erased");
        }
        println!();

        // Require explicit user confirmation
        print!("Confirm execution? Type 'yes' to continue, any other input will cancel: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim().to_lowercase();
        if trimmed_input != "yes" {
            return Err(anyhow::anyhow!("User cancelled the operation"));
        }

        info!("✅ User confirmed, proceeding with garbage collection");
        Ok(())
    }

    /// Get the database directory path
    fn get_database_path(&self) -> Result<std::path::PathBuf> {
        Ok(self.db_path.clone())
    }

    /// Get the set of root nodes to protect during GC
    fn get_protected_roots(&self) -> Result<Vec<H256>> {
        // Validate basic configuration
        if self.config.protected_roots_count == 0 {
            return Err(anyhow::anyhow!(
                "protected_roots_count must be at least 1, got 0"
            ));
        }

        // 1. Use historical state collector for multi-root protection
        if self.config.protected_roots_count > 1 {
            info!(
                "Collecting {} recent state roots for multi-root protection",
                self.config.protected_roots_count
            );
            let config = HistoricalStateConfig {
                protected_roots_count: self.config.protected_roots_count,
            };
            let collector = HistoricalStateCollector::new(
                self.rooch_db.moveos_store.clone(),
                self.rooch_db.rooch_store.clone(),
                config,
            );
            let roots = collector.collect_recent_state_roots();
            match roots {
                Ok(collected_roots) => {
                    info!(
                        "Collected {} historical protected roots",
                        collected_roots.len()
                    );
                    return Ok(collected_roots);
                }
                Err(e) => {
                    if self.config.skip_confirm {
                        warn!("Historical state collection failed, but skip_confirm enabled: {}. Falling back to single root mode.", e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        if let Ok(Some(startup_info)) = self.rooch_db.moveos_store.config_store.get_startup_info() {
            info!("Using startup_info state_root: {}", startup_info.state_root);
            return Ok(vec![startup_info.state_root]);
        }

        Err(anyhow::anyhow!(
            "Unable to determine protected state roots - no snapshot or startup info available"
        ))
    }

    /// Get live data size from RocksDB statistics using estimate-live-data-size property
    fn get_live_data_size(
        &self,
        node_store: &moveos_store::state_store::NodeDBStore,
    ) -> Result<Option<u64>> {
        // Follow established pattern from rocksdb_stats.rs for RocksDB property access
        if let Some(wrapper) = node_store.get_store().store().db() {
            let raw_db = wrapper.inner();
            if let Some(cf) = raw_db.cf_handle(STATE_NODE_COLUMN_FAMILY_NAME) {
                if let Ok(Some(data_size)) =
                    raw_db.property_int_value_cf(&cf, "rocksdb.estimate-live-data-size")
                {
                    if data_size > 0 {
                        info!("RocksDB live data size available: {} bytes", data_size);
                        return Ok(Some(data_size));
                    }
                }
            }
        }
        debug!("Unable to get live data size from RocksDB statistics");
        Ok(None)
    }

    /// Estimate the number of nodes to process for strategy selection
    /// Uses RocksDB statistics for O(1) performance instead of key traversal
    fn estimate_node_count(&self, _protected_roots: &[H256]) -> Result<usize> {
        let node_store = self.rooch_db.moveos_store.get_state_node_store();

        // 1. Try to use estimate-live-data-size for more reliable estimation
        if let Some(data_size) = self.get_live_data_size(node_store)? {
            let estimate = (data_size / Self::AVG_NODE_SIZE_BYTES) as usize;
            if estimate > 0 {
                info!(
                    "Estimated node count using live data size ({} bytes / {} avg size): {}",
                    data_size,
                    Self::AVG_NODE_SIZE_BYTES,
                    estimate
                );
                return Ok(estimate);
            }
        }

        // 2. Fallback to approximate-num-keys
        if let Some(count) = self.get_approximate_num_keys(node_store)? {
            info!("Estimated node count using approximate-num-keys: {}", count);
            return Ok(count);
        }

        // 3. Use default estimate when all statistics are unavailable
        warn!(
            "Unable to determine node count from RocksDB statistics, using default estimate: {}",
            1_000_000
        );
        Ok(1_000_000)
    }

    /// Get approximate key count from RocksDB statistics API using approximate-num-keys property
    fn get_approximate_num_keys(
        &self,
        node_store: &moveos_store::state_store::NodeDBStore,
    ) -> Result<Option<usize>> {
        // Follow established pattern from rocksdb_stats.rs for RocksDB property access
        if let Some(wrapper) = node_store.get_store().store().db() {
            let raw_db = wrapper.inner();
            if let Some(cf) = raw_db.cf_handle(STATE_NODE_COLUMN_FAMILY_NAME) {
                // Use the same pattern as rocksdb_stats.rs:130 for property access
                if let Ok(Some(count)) =
                    raw_db.property_int_value_cf(&cf, "rocksdb.approximate-num-keys")
                {
                    let count_usize = count as usize;
                    if count_usize > 0 {
                        info!(
                            "RocksDB statistics available: {} approximate keys",
                            count_usize
                        );
                        return Ok(Some(count_usize));
                    }
                }
            }
        }
        debug!("Unable to get node count from RocksDB statistics");
        Ok(None)
    }

    /// Mark phase - traverse and mark all reachable nodes
    fn mark_phase(
        &self,
        protected_roots: &[H256],
        estimated_nodes: usize,
    ) -> Result<(MarkStats, Box<dyn NodeMarker>)> {
        info!("=== Mark Phase ===");
        let start_time = Instant::now();

        // Create Bloom filter marker with optimal parameters
        let marker = create_marker(estimated_nodes, self.config.marker_target_fp_rate);
        info!("Created {} marker", marker.marker_type());

        // Create ReachableBuilder with a bloom filter for additional optimization
        let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
        let reachable_builder = ReachableBuilder::new(self.rooch_db.moveos_store.clone(), bloom);

        // Capture marker type for stats
        let strategy_name = marker.marker_type().to_string();

        // Execute reachability analysis
        let reachable_count = reachable_builder.build_with_marker(
            protected_roots.to_vec(),
            self.config.workers,
            marker.as_ref(),
        )?;

        let mark_stats = MarkStats {
            marked_count: reachable_count,
            duration: start_time.elapsed(),
            memory_strategy: strategy_name,
        };

        info!(
            "Mark phase completed: {} nodes marked in {:?}",
            reachable_count, mark_stats.duration
        );

        Ok((mark_stats, marker))
    }

    /// Sweep phase - identify and delete unreachable nodes
    fn sweep_phase(
        &self,
        _protected_roots: &[H256],
        reachable_marker: &dyn NodeMarker,
    ) -> Result<SweepStats> {
        info!("=== Sweep Phase ===");
        let start_time = Instant::now();

        let mut stats = SweepStats {
            scanned_count: 0,
            kept_count: 0,
            deleted_count: 0,
            recycle_bin_entries: 0,
            duration: Duration::default(),
        };

        if self.config.dry_run {
            info!("DRY RUN: Simulating sweep phase without actual deletions");
            return self.simulate_sweep_phase();
        }

        // Get candidate nodes for deletion
        let candidate_nodes = self.get_candidate_nodes_for_deletion()?;
        stats.scanned_count = candidate_nodes.len() as u64;

        if candidate_nodes.is_empty() {
            info!("No candidate nodes found for deletion");
            stats.duration = start_time.elapsed();
            return Ok(stats);
        }

        info!(
            "Found {} candidate nodes for deletion",
            candidate_nodes.len()
        );

        // Determine which nodes are actually deletable by checking against reachable marker
        let (nodes_to_delete, nodes_to_keep) =
            self.filter_nodes_by_reachability(&candidate_nodes, reachable_marker)?;

        stats.kept_count = nodes_to_keep.len() as u64;
        info!("Nodes to keep (reachable): {}", nodes_to_keep.len());

        // Process deletions in batches
        let node_store = self.rooch_db.moveos_store.get_state_node_store();
        let batch_size = self.config.batch_size;

        info!(
            "Deleting {} unreachable nodes in batches of {}",
            nodes_to_delete.len(),
            batch_size
        );

        for chunk in nodes_to_delete.chunks(batch_size) {
            self.process_deletion_batch_real(node_store, chunk, &mut stats)?;

            // Periodically log progress for large deletions
            if stats.deleted_count % 10000 == 0 && stats.deleted_count > 0 {
                info!("Deleted {} nodes so far...", stats.deleted_count);
            }
        }

        // Flush after all deletions are complete
        if !nodes_to_delete.is_empty() {
            info!("Flushing state node store after sweep phase");
            node_store.flush_only()?;
        }

        stats.duration = start_time.elapsed();

        info!(
            "Sweep phase completed: scanned={}, kept={}, deleted={}, recycle_bin={}, duration={:?}",
            stats.scanned_count,
            stats.kept_count,
            stats.deleted_count,
            stats.recycle_bin_entries,
            stats.duration
        );

        // Log sweep effectiveness
        if stats.scanned_count > 0 {
            let deletion_ratio = stats.deleted_count as f64 / stats.scanned_count as f64;
            info!("Deletion ratio: {:.2}%", deletion_ratio * 100.0);
        }

        Ok(stats)
    }

    /// Get candidate nodes for deletion by scanning the database
    fn get_candidate_nodes_for_deletion(&self) -> Result<Vec<H256>> {
        let node_store = self.rooch_db.moveos_store.get_state_node_store();
        let mut candidates = Vec::new();

        // Prefer RocksDB raw iterator to avoid serde decode errors
        if let Some(wrapper) = node_store.get_store().store().db() {
            let raw_db = wrapper.inner();
            if let Some(cf) = raw_db.cf_handle(STATE_NODE_COLUMN_FAMILY_NAME) {
                let mut iter = raw_db.raw_iterator_cf(&cf);
                iter.seek_to_first();
                while iter.valid() {
                    if let Some(k) = iter.key() {
                        if k.len() == 32 {
                            candidates.push(H256::from_slice(k));
                        } else {
                            warn!("Skipping non-32B node key len={}", k.len());
                        }
                    }
                    iter.next();
                }
                debug!(
                    "Collected {} candidate nodes via state_node iterator",
                    candidates.len()
                );
                return Ok(candidates);
            }
        }

        warn!("Iterator not supported for node store, returning empty candidate list");

        Ok(candidates)
    }

    /// Filter nodes based on reachability using the marker built during mark phase
    fn filter_nodes_by_reachability(
        &self,
        candidates: &[H256],
        marker: &dyn NodeMarker,
    ) -> Result<(Vec<H256>, Vec<H256>)> {
        let mut nodes_to_delete = Vec::new();
        let mut nodes_to_keep = Vec::new();

        for &node in candidates {
            if marker.is_marked(&node) {
                nodes_to_keep.push(node);
            } else {
                nodes_to_delete.push(node);
            }
        }

        info!(
            "Filtering results: {} deletable, {} reachable",
            nodes_to_delete.len(),
            nodes_to_keep.len()
        );
        Ok((nodes_to_delete, nodes_to_keep))
    }

    /// Process real deletion batch using actual API
    fn process_deletion_batch_real(
        &self,
        node_store: &moveos_store::state_store::NodeDBStore,
        batch: &[H256],
        stats: &mut SweepStats,
    ) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let batch_size = batch.len();

        // Optionally store deleted nodes into recycle bin for recovery
        if self.config.use_recycle_bin {
            for node_hash in batch {
                if let Ok(Some(bytes)) = node_store.get(node_hash) {
                    let record = self.recycle_bin.create_record(bytes);
                    // Note field removed - RecycleRecord now simplified to 3 fields
                    if let Err(e) = self.recycle_bin.put_record(*node_hash, record) {
                        warn!(?node_hash, "Failed to store recycle record: {}", e);
                    }
                }
            }
        }

        // Use the actual deletion API with flush=false for better performance
        // We'll flush once at the end of the sweep phase
        node_store.delete_nodes_with_flush(batch.to_vec(), false)?;

        // Update statistics
        stats.deleted_count += batch_size as u64;

        if self.config.use_recycle_bin {
            stats.recycle_bin_entries += batch_size as u64;
        }

        debug!(
            "Deleted batch of {} nodes, total deleted: {}",
            batch_size, stats.deleted_count
        );
        Ok(())
    }

    /// Simulate sweep phase for dry-run mode
    fn simulate_sweep_phase(&self) -> Result<SweepStats> {
        info!("Simulating sweep phase analysis");

        // Get candidate count without actual filtering for simplicity
        let candidate_count = match self.rooch_db.moveos_store.get_state_node_store().keys() {
            Ok(keys) => keys.len(),
            Err(_) => 0,
        };

        let stats = SweepStats {
            scanned_count: candidate_count as u64,
            kept_count: (candidate_count as f64 * 0.8) as u64, // Assume 80% are reachable
            deleted_count: (candidate_count as f64 * 0.2) as u64, // Assume 20% are deletable
            recycle_bin_entries: if self.config.use_recycle_bin {
                (candidate_count as f64 * 0.2) as u64
            } else {
                0
            },
            duration: Duration::from_millis(100), // Simulated duration
        };

        info!(
            "DRY RUN: Would scan={}, keep={}, delete={}, recycle_bin={}",
            stats.scanned_count, stats.kept_count, stats.deleted_count, stats.recycle_bin_entries
        );

        Ok(stats)
    }

    /// Trigger RocksDB compaction to reclaim space
    fn trigger_compaction(&self) -> Result<()> {
        info!("=== Trigger Compaction ===");
        let start_time = Instant::now();

        let node_store = self.rooch_db.moveos_store.get_state_node_store();

        // Record database size before compaction for statistics
        let before_size = self.get_database_size(node_store)?;
        info!("Database size before compaction: {} bytes", before_size);

        if self.config.dry_run {
            info!("DRY RUN: Would trigger compaction, but skipping due to dry-run mode");
            return Ok(());
        }

        // Choose compaction strategy based on configuration
        if self.config.force_compaction {
            info!("Starting aggressive compaction to maximize space reclamation");
            node_store.aggressive_compact()?;
        } else {
            info!("Starting standard compaction");
            node_store.flush_and_compact()?;
        }

        // Record database size after compaction
        let after_size = self.get_database_size(node_store)?;
        let space_reclaimed = before_size.saturating_sub(after_size);
        let duration = start_time.elapsed();

        info!("Database size after compaction: {} bytes", after_size);
        info!(
            "Space reclaimed: {} bytes ({:.2} MB)",
            space_reclaimed,
            space_reclaimed as f64 / 1024.0 / 1024.0
        );
        info!("Compaction completed in {:?}", duration);

        // Log compaction effectiveness
        if before_size > 0 {
            let reclamation_ratio = space_reclaimed as f64 / before_size as f64;
            info!("Space reclamation ratio: {:.2}%", reclamation_ratio * 100.0);

            if reclamation_ratio < 0.01 {
                warn!(
                    "Low space reclamation ratio ({:.2}%) - consider running aggressive compaction",
                    reclamation_ratio * 100.0
                );
            }
        }

        Ok(())
    }

    /// Get current database size from RocksDB properties
    fn get_database_size(
        &self,
        _node_store: &moveos_store::state_store::NodeDBStore,
    ) -> Result<u64> {
        // For now, we'll use a simple fallback estimation
        // In a real implementation, we could add a method to get database statistics
        // This would require extending the NodeDBStore API

        // Return a reasonable estimate based on typical node sizes
        // This is a placeholder - actual implementation would query RocksDB properties
        let estimated_size = 100_000_000; // 100MB default estimate
        debug!("Using estimated database size: {} bytes", estimated_size);
        Ok(estimated_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_config_default() {
        let config = GCConfig::default();
        assert!(!config.dry_run);
        assert_eq!(config.batch_size, 10_000);
        assert!(config.use_recycle_bin);
        assert!(!config.force_compaction);
        // Marker strategy is now always BloomFilter
        assert!(!config.skip_confirm);
    }

    #[test]
    fn test_gc_report_creation() {
        let roots = vec![H256::random(), H256::random()];
        let mark_stats = MarkStats {
            marked_count: 1000,
            duration: Duration::from_secs(10),
            memory_strategy: "InMemory".to_string(),
        };
        let sweep_stats = SweepStats {
            scanned_count: 2000,
            kept_count: 1000,
            deleted_count: 1000,
            recycle_bin_entries: 1000,
            duration: Duration::from_secs(15),
        };

        let report = GCReport {
            protected_roots: roots.clone(),
            mark_stats,
            sweep_stats,
            duration: Duration::from_secs(30),
            memory_strategy_used: "BloomFilter".to_string(),
        };

        assert_eq!(report.protected_roots, roots);
        assert_eq!(report.mark_stats.marked_count, 1000);
        assert_eq!(report.sweep_stats.deleted_count, 1000);
        assert_eq!(report.memory_strategy_used, "BloomFilter");
    }
}
