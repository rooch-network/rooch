// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use metrics::RegistryService;
use moveos_store::prune::PruneStore;
use raw_store::SchemaStore;
use rooch_config::RoochOpt;
use rooch_store::state_store::StateStore;
use rooch_types::error::RoochResult;
use std::sync::Arc;

/// Diagnose pruner state and identify why space is not reclaimed
#[derive(Parser, Debug)]
pub struct PruneDiagnosisCommand {
    /// Number of recent state roots to analyze
    #[clap(long, default_value = "1000")]
    pub sample_size: u64,
}

#[async_trait]
impl CommandAction<String> for PruneDiagnosisCommand {
    async fn execute(self) -> RoochResult<String> {
        let result = self
            .execute_impl()
            .map_err(|e| rooch_types::error::RoochError::from(e))?;
        Ok(result)
    }
}

impl PruneDiagnosisCommand {
    fn execute_impl(self) -> Result<String> {
        let opt = RoochOpt::new_with_default(None, None, None)?;
        let registry_service = RegistryService::default();
        let rooch_db = rooch_db::RoochDB::init(opt.store_config(), &registry_service.default_registry())?;
        let moveos_store = Arc::new(rooch_db.moveos_store);
        let rooch_store = Arc::new(rooch_db.rooch_store);

        let mut out = String::new();
        out.push_str("=== Pruner Diagnosis ===\n\n");

        // 1. Check deleted_state_root_bloom
        let deleted_bloom = moveos_store
            .load_deleted_state_root_bloom()
            .ok()
            .flatten();
        
        if deleted_bloom.is_some() {
            out.push_str("Deleted State Root Bloom: EXISTS\n");
        } else {
            out.push_str("Deleted State Root Bloom: NOT FOUND\n");
        }

        // 2. Check reachable bloom
        let reachable_bloom = moveos_store
            .load_prune_meta_bloom()
            .ok()
            .flatten();
        
        if reachable_bloom.is_some() {
            out.push_str("Reachable Bloom: EXISTS\n");
        } else {
            out.push_str("Reachable Bloom: NOT FOUND\n");
        }

        // 3. Sample recent state roots and check their status
        out.push_str(&format!("\n=== Recent {} State Roots Analysis ===\n", self.sample_size));
        
        let snapshot = moveos_store
            .load_prune_meta_snapshot()
            .ok()
            .flatten()
            .unwrap_or_default();
        
        let latest_order = snapshot.latest_order;
        out.push_str(&format!("Latest processed order: {}\n", latest_order));

        let start_order = latest_order.saturating_sub(self.sample_size);
        let mut sampled = 0;
        let mut already_deleted = 0;
        let mut root_nodes_reachable = 0;

        for order in (start_order..=latest_order).rev().take(self.sample_size as usize) {
            if let Some(scs) = rooch_store.get_state_change_set(order).ok().flatten() {
                sampled += 1;
                let root = scs.state_change_set.state_root;

                // Check if marked as deleted
                if let Some(ref bloom) = deleted_bloom {
                    if bloom.contains(&root) {
                        already_deleted += 1;
                    }
                }

                // Check if root itself is reachable
                if let Some(ref bloom) = reachable_bloom {
                    if bloom.contains(&root) {
                        root_nodes_reachable += 1;
                    }
                }
            }
        }

        out.push_str(&format!("\nSampled {} roots from order {} to {}\n", sampled, start_order, latest_order));
        out.push_str(&format!("  Already marked deleted: {} ({:.1}%)\n", already_deleted, already_deleted as f64 / sampled as f64 * 100.0));
        out.push_str(&format!("  Root marked as reachable: {} ({:.1}%)\n", root_nodes_reachable, root_nodes_reachable as f64 / sampled as f64 * 100.0));

        // 4. Estimate deletion effectiveness
        if already_deleted > 0 {
            out.push_str(&format!("\n⚠️  WARNING: {} roots already marked deleted but may not be physically removed\n", already_deleted));
            out.push_str("   Possible causes:\n");
            out.push_str("   1. Tombstones still in memtable (not flushed)\n");
            out.push_str("   2. Insufficient compaction to merge tombstones\n");
            out.push_str("   3. Shared nodes still referenced by other roots\n");
        }

        // 5. Check actual node store stats
        if let Some(wrapper) = moveos_store.node_store.get_store().store().db() {
            let raw = wrapper.inner();
            if let Some(cf) = raw.cf_handle("state_node") {
                if let Ok(Some(total)) = raw.property_int_value_cf(&cf, "rocksdb.total-sst-files-size") {
                    if let Ok(Some(live)) = raw.property_int_value_cf(&cf, "rocksdb.live-sst-files-size") {
                        let dead = total.saturating_sub(live);
                        out.push_str("\n=== State Node SST Stats ===\n");
                        out.push_str(&format!("Total SST: {:.2} GB\n", total as f64 / 1e9));
                        out.push_str(&format!("Live SST:  {:.2} GB\n", live as f64 / 1e9));
                        out.push_str(&format!("Dead SST:  {:.2} GB ({:.1}%)\n", dead as f64 / 1e9, dead as f64 / total as f64 * 100.0));
                        
                        if dead == 0 {
                            out.push_str("\n⚠️  Dead SST is 0: deletions not effective or fully compacted\n");
                        }
                    }
                }

                if let Ok(Some(pending)) = raw.property_int_value_cf(&cf, "rocksdb.estimate-pending-compaction-bytes") {
                    out.push_str(&format!("\nPending compaction: {:.2} GB\n", pending as f64 / 1e9));
                    if pending > 100_000_000 {
                        out.push_str("  → Significant compaction backlog, run GC to compact\n");
                    }
                }
            }
        }

        // 6. Recommendations
        out.push_str("\n=== Recommendations ===\n");
        if already_deleted > sampled / 2 {
            out.push_str("1. Most roots already marked deleted - run: rooch db rocksdb-gc\n");
            out.push_str("2. If GC shows no reclaim, check if nodes are shared across roots\n");
        }
        if root_nodes_reachable > sampled / 2 {
            out.push_str("3. Many roots marked reachable - check BuildReach logic\n");
        }
        out.push_str("4. Use: rooch db rocksdb-stats to view detailed RocksDB metrics\n");

        Ok(out)
    }
}
