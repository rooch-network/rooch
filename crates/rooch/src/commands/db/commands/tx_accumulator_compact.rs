// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::db::commands::{load_accumulator, open_rocks};
use crate::utils::open_rooch_db;
use accumulator::node_index::{FrozenSubTreeIterator, NodeIndex};
use accumulator::{Accumulator, AccumulatorNode, AccumulatorTreeStore as _};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::{ACCUMULATOR_PLACEHOLDER_HASH, H256};
use rooch_config::R_OPT_NET_HELP;
use rooch_store::{RoochStore, TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME};
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use std::time::Instant;

/// Compact historical non-frozen transaction accumulator nodes.
///
/// The command replays historical leaves and reconstructs the transient
/// non-frozen internal node hashes that older versions persisted. Frozen
/// subtrees and leaves are not selected for deletion.
#[derive(Debug, Parser)]
pub struct TxAccumulatorCompactCommand {
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    /// First leaf index to include in deletion candidate collection.
    /// Earlier leaves are still replayed to rebuild frozen roots.
    #[clap(long, default_value_t = 0)]
    pub start_index: u64,

    /// Stop before this leaf index. Defaults to the current accumulator leaf count.
    #[clap(long)]
    pub end_index: Option<u64>,

    /// Number of candidate hashes to check/delete per batch.
    #[clap(long, default_value_t = 10000)]
    pub batch_size: usize,

    /// Print progress after this many replayed leaves. 0 disables progress lines.
    #[clap(long, default_value_t = 1_000_000)]
    pub progress_interval: u64,

    /// Actually delete matched non-frozen nodes. Without this flag the command is a dry-run.
    #[clap(long)]
    pub execute: bool,

    /// Force RocksDB compaction on the transaction_acc_node column family after deletion.
    #[clap(long)]
    pub force_compaction: bool,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct CompactReport {
    start_index: u64,
    end_index: u64,
    replayed_leaves: u64,
    candidate_nodes: u64,
    existing_nodes: u64,
    deleted_nodes: u64,
    dry_run: bool,
}

#[async_trait]
impl CommandAction<String> for TxAccumulatorCompactCommand {
    async fn execute(self) -> RoochResult<String> {
        self.execute_impl().map_err(Into::into)
    }
}

impl TxAccumulatorCompactCommand {
    fn execute_impl(self) -> Result<String> {
        ensure!(self.batch_size > 0, "batch-size must be greater than 0");

        let started_at = Instant::now();
        let dry_run = !self.execute;
        let (_root, rooch_db, _start_time) =
            open_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());
        let rooch_store = rooch_db.rooch_store.clone();
        let (tx_accumulator, _last_order) = load_accumulator(rooch_store.clone())?;
        let leaf_count = tx_accumulator.num_leaves();
        let end_index = self.end_index.unwrap_or(leaf_count);
        ensure!(
            self.start_index <= end_index && end_index <= leaf_count,
            "invalid range [{}, {}), current leaf count {}",
            self.start_index,
            end_index,
            leaf_count
        );

        let mut replayer = NonFrozenNodeReplayer::new();
        let mut report = CompactReport {
            start_index: self.start_index,
            end_index,
            dry_run,
            ..Default::default()
        };
        let mut pending_hashes = Vec::with_capacity(self.batch_size);
        let mut out = String::new();

        for leaf_index in 0..end_index {
            let leaf = tx_accumulator.get_leaf(leaf_index)?.ok_or_else(|| {
                anyhow::anyhow!("transaction accumulator leaf {} not found", leaf_index)
            })?;
            let non_frozen_hashes = replayer.append_one(leaf)?;
            if leaf_index < self.start_index {
                continue;
            }

            report.replayed_leaves += 1;
            report.candidate_nodes += non_frozen_hashes.len() as u64;
            pending_hashes.extend(non_frozen_hashes);
            if pending_hashes.len() >= self.batch_size {
                flush_candidates(&rooch_store, &mut pending_hashes, dry_run, &mut report)?;
            }

            if self.progress_interval > 0 && report.replayed_leaves % self.progress_interval == 0 {
                writeln!(
                    out,
                    "progress: replayed={} candidates={} existing={} deleted={}",
                    report.replayed_leaves,
                    report.candidate_nodes,
                    report.existing_nodes,
                    report.deleted_nodes
                )?;
            }
        }
        flush_candidates(&rooch_store, &mut pending_hashes, dry_run, &mut report)?;

        drop(tx_accumulator);
        drop(rooch_store);
        drop(rooch_db);

        let compact_elapsed = if self.force_compaction && !dry_run {
            Some(compact_tx_accumulator_cf(
                self.base_data_dir,
                self.chain_id,
            )?)
        } else {
            None
        };

        writeln!(out, "=== Tx Accumulator Compact Result ===")?;
        writeln!(out, "mode: {}", if dry_run { "dry-run" } else { "execute" })?;
        writeln!(out, "range: [{}..{})", report.start_index, report.end_index)?;
        writeln!(out, "replayed leaves: {}", report.replayed_leaves)?;
        writeln!(
            out,
            "candidate non-frozen nodes: {}",
            report.candidate_nodes
        )?;
        writeln!(out, "existing candidate nodes: {}", report.existing_nodes)?;
        writeln!(out, "deleted nodes: {}", report.deleted_nodes)?;
        if let Some(elapsed) = compact_elapsed {
            writeln!(out, "rocksdb compaction: {:?}", elapsed)?;
        }
        writeln!(out, "elapsed: {:?}", started_at.elapsed())?;
        Ok(out)
    }
}

fn flush_candidates(
    rooch_store: &RoochStore,
    pending_hashes: &mut Vec<H256>,
    dry_run: bool,
    report: &mut CompactReport,
) -> Result<()> {
    if pending_hashes.is_empty() {
        return Ok(());
    }

    pending_hashes.sort();
    pending_hashes.dedup();
    let existing_hashes = rooch_store
        .transaction_accumulator_store
        .multiple_get(pending_hashes.clone())?
        .into_iter()
        .zip(pending_hashes.iter())
        .filter_map(|(node, hash)| node.map(|_| *hash))
        .collect::<Vec<_>>();

    report.existing_nodes += existing_hashes.len() as u64;
    if !dry_run && !existing_hashes.is_empty() {
        let deleted = existing_hashes.len() as u64;
        rooch_store
            .transaction_accumulator_store
            .delete_nodes(existing_hashes)?;
        report.deleted_nodes += deleted;
    }

    pending_hashes.clear();
    Ok(())
}

fn compact_tx_accumulator_cf(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> Result<std::time::Duration> {
    let db = open_rocks(base_data_dir, chain_id)?;
    let raw = db.inner();
    let cf = raw
        .cf_handle(TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME)
        .ok_or_else(|| anyhow::anyhow!("transaction accumulator column family not found"))?;

    raw.flush_wal(true)?;
    raw.flush_cf(&cf)?;
    use rocksdb::{BottommostLevelCompaction, CompactOptions};
    let mut copt = CompactOptions::default();
    copt.set_bottommost_level_compaction(BottommostLevelCompaction::Force);
    copt.set_exclusive_manual_compaction(true);

    let start = Instant::now();
    raw.compact_range_cf_opt(&cf, None::<&[u8]>, None::<&[u8]>, &copt);
    Ok(start.elapsed())
}

#[derive(Debug, Default)]
struct NonFrozenNodeReplayer {
    num_leaves: u64,
    frozen_roots: HashMap<NodeIndex, H256>,
}

impl NonFrozenNodeReplayer {
    fn new() -> Self {
        Self::default()
    }

    fn append_one(&mut self, leaf: H256) -> Result<Vec<H256>> {
        let leaf_pos = NodeIndex::from_leaf_index(self.num_leaves);
        let last_new_leaf_count = self.num_leaves + 1;
        let root_level = NodeIndex::root_level_from_leaf_count(last_new_leaf_count);
        let mut new_frozen = HashMap::new();

        let mut pos = leaf_pos;
        let mut hash = leaf;
        new_frozen.insert(pos, hash);

        while pos.is_right_child() {
            let sibling = pos.sibling();
            let left_hash = new_frozen
                .get(&sibling)
                .or_else(|| self.frozen_roots.get(&sibling))
                .copied()
                .ok_or_else(|| anyhow::anyhow!("missing frozen sibling {:?}", sibling))?;
            let internal_node = AccumulatorNode::new_internal(pos.parent(), left_hash, hash);
            hash = internal_node.hash();
            pos = pos.parent();
            new_frozen.insert(pos, hash);
        }

        let mut non_frozen_hashes = Vec::new();
        for _ in pos.level()..root_level {
            let internal_node = if pos.is_left_child() {
                AccumulatorNode::new_internal(pos.parent(), hash, *ACCUMULATOR_PLACEHOLDER_HASH)
            } else {
                let sibling = pos.sibling();
                let left_hash = self
                    .frozen_roots
                    .get(&sibling)
                    .copied()
                    .unwrap_or(*ACCUMULATOR_PLACEHOLDER_HASH);
                AccumulatorNode::new_internal(pos.parent(), left_hash, hash)
            };
            hash = internal_node.hash();
            pos = pos.parent();
            non_frozen_hashes.push(hash);
        }

        self.num_leaves = last_new_leaf_count;
        self.frozen_roots = FrozenSubTreeIterator::new(self.num_leaves)
            .map(|index| {
                let hash = new_frozen
                    .get(&index)
                    .or_else(|| self.frozen_roots.get(&index))
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("missing frozen root {:?}", index))?;
                Ok((index, hash))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(non_frozen_hashes)
    }
}

#[cfg(test)]
fn replay_non_frozen_hashes(leaves: &[H256]) -> Result<Vec<H256>> {
    let mut replayer = NonFrozenNodeReplayer::new();
    let mut hashes = Vec::new();
    for leaf in leaves {
        hashes.extend(replayer.append_one(*leaf)?);
    }
    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use accumulator::MerkleAccumulator;

    #[test]
    fn test_replay_non_frozen_hashes_for_single_leaf_appends() {
        let leaves = (0..8).map(|_| H256::random()).collect::<Vec<_>>();
        let hashes = replay_non_frozen_hashes(&leaves).unwrap();

        // Perfect tree sizes have no non-frozen root after the last append, but
        // intermediate non-perfect prefixes still produce transient nodes.
        assert!(!hashes.is_empty());
        assert_eq!(hashes.len(), 10);
    }

    #[test]
    fn test_flush_candidates_deletes_only_existing_nodes() {
        let (rooch_store, _tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let accumulator =
            MerkleAccumulator::new_empty(rooch_store.get_transaction_accumulator_store());
        let leaves = (0..6).map(|_| H256::random()).collect::<Vec<_>>();
        for leaf in &leaves {
            accumulator.append(&[*leaf]).unwrap();
            accumulator.flush().unwrap();
        }

        let old_non_frozen_hashes = replay_non_frozen_hashes(&leaves).unwrap();
        let persisted_old_nodes = old_non_frozen_hashes
            .iter()
            .map(|hash| AccumulatorNode::new_leaf(NodeIndex::from_leaf_index(10_000), *hash))
            .collect::<Vec<_>>();
        rooch_store
            .transaction_accumulator_store
            .save_nodes(persisted_old_nodes)
            .unwrap();

        let mut dry_run_report = CompactReport::default();
        let mut pending = old_non_frozen_hashes.clone();
        flush_candidates(&rooch_store, &mut pending, true, &mut dry_run_report).unwrap();
        assert_eq!(
            dry_run_report.existing_nodes,
            old_non_frozen_hashes.len() as u64
        );
        assert_eq!(dry_run_report.deleted_nodes, 0);

        let mut execute_report = CompactReport::default();
        let mut pending = old_non_frozen_hashes.clone();
        flush_candidates(&rooch_store, &mut pending, false, &mut execute_report).unwrap();
        assert_eq!(
            execute_report.deleted_nodes,
            old_non_frozen_hashes.len() as u64
        );

        let remaining = rooch_store
            .transaction_accumulator_store
            .multiple_get(old_non_frozen_hashes)
            .unwrap();
        assert!(remaining.into_iter().all(|node| node.is_none()));
    }
}
