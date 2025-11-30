// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_inner_rocks;
use crate::utils::open_rooch_db_readonly;
use clap::Parser;
use moveos_store::prune::PruneStore;
use moveos_types::h256::H256;
use rooch_config::{RoochOpt, R_OPT_NET_HELP};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use smt::jellyfish_merkle::node_type::Node;
use smt::NodeReader;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::PathBuf;

/// List smt_stale entries, optionally filtered by tx_order range.
#[derive(Debug, Parser)]
pub struct ListStaleCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: BuiltinChainID,
    /// Inclusive lower bound of tx_order to include
    #[clap(long)]
    pub min_order: Option<u64>,
    /// Inclusive upper bound of tx_order to include
    #[clap(long)]
    pub max_order: Option<u64>,
    /// Max sample rows to print
    #[clap(long, default_value_t = 50)]
    pub limit: usize,
    /// Also check whether stale node hashes are reachable in the latest snapshot
    #[clap(long)]
    pub check_reach: bool,
    /// Limit of nodes to scan for reachability (0 = full)
    #[clap(long, default_value_t = 0)]
    pub scan_limit: usize,
}

impl ListStaleCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let opt = RoochOpt::new_with_default(
            self.base_data_dir.clone(),
            Some(RoochChainID::Builtin(self.chain_id)),
            None,
        )
        .map_err(|e| RoochError::CommandArgumentError(e.to_string()))?;

        let store_dir = opt.store_config().get_store_dir();
        let db = open_inner_rocks(
            store_dir
                .to_str()
                .ok_or_else(|| RoochError::CommandArgumentError("invalid store dir".to_owned()))?,
            vec!["smt_stale".to_string()],
            true,
        )
        .map_err(|e| RoochError::UnexpectedError(e.to_string()))?;

        // Optional: build reachable set from latest snapshot
        let reachable = if self.check_reach {
            let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
                self.base_data_dir.clone(),
                Some(RoochChainID::Builtin(self.chain_id)),
            );
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
                        latest_order: si.size,
                    }
                });
            Some(build_reachable(
                &rooch_db.moveos_store,
                snapshot.state_root,
                self.scan_limit,
            )?)
        } else {
            None
        };

        let cf = db
            .cf_handle("smt_stale")
            .ok_or_else(|| RoochError::UnexpectedError("missing smt_stale cf".to_owned()))?;

        let mut iter = db.raw_iterator_cf(cf);
        iter.seek_to_first();

        let mut total_in_range = 0usize;
        let mut weird_len = 0usize;
        let mut samples: Vec<(u64, String)> = Vec::new();
        let mut orders: BTreeMap<u64, usize> = BTreeMap::new();
        let mut reachable_hits: Vec<(u64, String)> = Vec::new();
        let mut reachable_count = 0usize;

        while iter.valid() {
            if let Some(k) = iter.key() {
                if k.len() == 64 {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(&k[24..32]);
                    let order = u64::from_be_bytes(buf);
                    let in_min = self.min_order.map(|m| order >= m).unwrap_or(true);
                    let in_max = self.max_order.map(|m| order <= m).unwrap_or(true);
                    if in_min && in_max {
                        total_in_range += 1;
                        *orders.entry(order).or_insert(0) += 1;
                        let node_hash = H256::from_slice(&k[32..64]);
                        if samples.len() < self.limit {
                            samples.push((order, hex::encode(&k[32..64])));
                        }
                        if let Some(reach) = reachable.as_ref() {
                            if reach.contains(&node_hash) {
                                reachable_count += 1;
                                if reachable_hits.len() < self.limit {
                                    reachable_hits.push((order, hex::encode(&k[32..64])));
                                }
                            }
                        }
                    }
                } else {
                    weird_len += 1;
                }
            }
            iter.next();
        }

        let mut out = String::new();
        use std::fmt::Write as _;
        writeln!(
            out,
            "smt_stale entries in range: {} (weird len: {})",
            total_in_range, weird_len
        )
        .ok();
        writeln!(out, "orders (count, showing all present orders in range):").ok();
        for (o, c) in orders.iter() {
            writeln!(out, "  order {} => {} entries", o, c).ok();
        }
        writeln!(out, "samples (up to {}): order | node_hash", self.limit).ok();
        for (o, h) in samples {
            writeln!(out, "  {} | {}", o, h).ok();
        }

        if let Some(reach) = reachable {
            writeln!(
                out,
                "reachable_in_stale: {} (up to {} shown)",
                reachable_count, self.limit
            )
            .ok();
            for (o, h) in reachable_hits {
                writeln!(out, "  {} | {}", o, h).ok();
            }
            let _ = reach;
        }

        Ok(out)
    }
}

fn build_reachable(
    store: &moveos_store::MoveOSStore,
    root: moveos_types::h256::H256,
    scan_limit: usize,
) -> RoochResult<HashSet<moveos_types::h256::H256>> {
    let mut set = HashSet::new();
    let node_store = store.get_state_node_store();
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
            if let Some(child_root) = try_extract_child_root(&bytes) {
                stack.push(child_root);
            } else if let Ok(smt::jellyfish_merkle::node_type::Node::Internal(internal)) =
                Node::<moveos_types::h256::H256, Vec<u8>>::decode(&bytes)
            {
                for child in internal.all_child() {
                    stack.push(child.into());
                }
            }
        }
    }
    Ok(set)
}

fn try_extract_child_root(bytes: &[u8]) -> Option<moveos_types::h256::H256> {
    use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;
    let node =
        Node::<moveos_types::state::FieldKey, moveos_types::state::ObjectState>::decode(bytes)
            .ok()?;
    if let Node::Leaf(leaf) = node {
        let state = &leaf.value().origin;
        if let Some(hash) = state.metadata.state_root {
            if hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                return Some(hash);
            }
        }
    }
    None
}
