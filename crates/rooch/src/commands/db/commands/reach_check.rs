// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use clap::Parser;
use moveos_store::prune::PruneStore;
use moveos_types::h256::H256;
use rooch_types::error::RoochResult;
use smt::jellyfish_merkle::node_type::Node;
use smt::NodeReader;
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;
use std::collections::HashSet;

/// Offline reachability check: runs the same DFS as SweepExpired on the latest snapshot,
/// then reports whether given node hashes are reachable.
#[derive(Debug, Parser)]
pub struct ReachCheckCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<std::path::PathBuf>,
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,
    /// Node hashes to check (0x…)
    #[clap(long = "hash")]
    pub hashes: Vec<String>,
    /// Override state_root to traverse (default: latest snapshot/root)
    #[clap(long)]
    pub root: Option<String>,
    /// Limit of nodes to scan (0 = full)
    #[clap(long, default_value_t = 0)]
    pub scan_limit: usize,
    /// Additionally scan and print child_root occurrences (table roots) and samples
    #[clap(long)]
    pub scan_child_roots: bool,
    /// Dump first N leaf nodes with their state_root (for debugging)
    #[clap(long, default_value_t = 0)]
    pub dump_leaves: usize,
}

impl ReachCheckCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        // Load latest snapshot from prune meta; fall back to latest startup info
        let snapshot = rooch_db
            .moveos_store
            .load_prune_meta_snapshot()?
            .unwrap_or_else(|| {
                let si = rooch_db
                    .moveos_store
                    .config_store
                    .get_startup_info()
                    .expect("startup info")
                    .expect("startup info should exist");
                moveos_types::prune::PruneSnapshot {
                    state_root: si.state_root,
                    latest_order: si.size, // best-effort fallback
                }
            });

        let state_root = if let Some(root_hex) = self.root.as_ref() {
            parse_h256(root_hex).expect("invalid root hex")
        } else {
            snapshot.state_root
        };

        // Run DFS to build reachable set. We reuse StateDBStore's iterators to traverse.
        let mut leaf_samples = Vec::new();
        let reachable = build_reachable(
            &rooch_db.moveos_store,
            state_root,
            self.scan_limit,
            self.scan_child_roots,
            self.dump_leaves,
            &mut leaf_samples,
        )?;

        let mut out = String::new();
        use std::fmt::Write as _;
        writeln!(
            out,
            "ReachCheck snapshot: root={:#x} latest_order={}",
            state_root, snapshot.latest_order
        )
        .ok();

        for hstr in &self.hashes {
            let hh = parse_h256(hstr)?;
            let found = reachable.contains(&hh);
            writeln!(out, "{:#x} => reachable={}", hh, found).ok();
        }

        // Summary
        writeln!(out, "Scanned nodes: {}", reachable.len()).ok();
        if self.dump_leaves > 0 {
            writeln!(out, "Leaf samples (up to {}):", self.dump_leaves).ok();
            for (i, (k, sr)) in leaf_samples.iter().enumerate() {
                let sr_str = sr
                    .map(|h| format!("{:#x}", h))
                    .unwrap_or_else(|| "None/placeholder".to_string());
                writeln!(out, "{:04}: key={:#x} state_root={}", i, k, sr_str).ok();
            }
        }
        Ok(out)
    }
}

fn parse_h256(s: &str) -> RoochResult<H256> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(H256(arr))
}

fn build_reachable(
    store: &moveos_store::MoveOSStore,
    root: H256,
    scan_limit: usize,
    track_child_roots: bool,
    dump_leaves: usize,
    leaf_samples: &mut Vec<(H256, Option<H256>)>,
) -> RoochResult<HashSet<H256>> {
    let mut set = HashSet::new();
    let node_store = store.get_state_node_store();
    let mut child_roots = Vec::new();
    let mut stack = vec![root];
    while let Some(node_hash) = stack.pop() {
        if set.contains(&node_hash) {
            continue;
        }
        set.insert(node_hash);
        if scan_limit > 0 && set.len() >= scan_limit {
            break;
        }
        if let Some(bytes) = node_store.get(&node_hash)? {
            if let Some((child_root, state_root)) = try_extract_child_root(&bytes) {
                stack.push(child_root);
                if track_child_roots {
                    child_roots.push(child_root);
                }
                if dump_leaves > 0 && leaf_samples.len() < dump_leaves {
                    leaf_samples.push((node_hash, state_root));
                }
            } else if let Ok(smt::jellyfish_merkle::node_type::Node::Internal(internal)) =
                smt::jellyfish_merkle::node_type::Node::<H256, Vec<u8>>::decode(&bytes)
            {
                for child in internal.all_child() {
                    stack.push(child.into());
                }
            }
        }
    }
    if track_child_roots {
        tracing::info!(
            "Detected child_roots: count={} sample={:?}",
            child_roots.len(),
            child_roots
                .iter()
                .take(10)
                .map(|h| format!("{:#x}", h))
                .collect::<Vec<_>>()
        );
    }
    Ok(set)
}

/// Decode leaf node and extract (child_root/state_root) if present.
fn try_extract_child_root(bytes: &[u8]) -> Option<(H256, Option<H256>)> {
    let node = Node::<H256, moveos_types::state::ObjectState>::decode(bytes).ok()?;
    if let Node::Leaf(leaf) = node {
        let state = &leaf.value().origin;
        if let Some(hash) = state.metadata.state_root {
            if hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                return Some((hash, state.metadata.state_root));
            }
        }
    }
    None
}
