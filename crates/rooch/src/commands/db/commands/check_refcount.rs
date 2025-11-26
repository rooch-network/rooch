// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_inner_rocks;
use clap::Parser;
use rooch_config::{RoochOpt, R_OPT_NET_HELP};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use std::collections::HashSet;
use std::path::PathBuf;

/// Compare state_node vs node_refcount and exclude smt_stale entries.
#[derive(Debug, Parser)]
pub struct CheckRefcountCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: BuiltinChainID,
    /// Sample count to print for mismatched hashes
    #[clap(long, default_value_t = 10)]
    pub sample: usize,
}

impl CheckRefcountCommand {
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
            vec![
                "state_node".to_string(),
                "node_refcount".to_string(),
                "smt_stale".to_string(),
            ],
            true,
        )
        .map_err(|e| RoochError::UnexpectedError(e.to_string()))?;

        let (state32, state_weird) = collect_keys(&db, "state_node", 32)?;
        let (refc32, refc_weird) = collect_keys(&db, "node_refcount", 32)?;

        // smt_stale key = (tx_order 32B || node_hash 32B)
        let (stale_nodes, stale_weird, stale_order_map) = {
            let cf = db
                .cf_handle("smt_stale")
                .ok_or_else(|| RoochError::UnexpectedError("missing smt_stale cf".to_owned()))?;
            let mut iter = db.raw_iterator_cf(cf);
            iter.seek_to_first();
            let mut nodes = HashSet::new();
            let mut weird = 0usize;
            // Map node_hash -> (count, min_order, max_order) for diagnostics
            let mut order_map = std::collections::HashMap::new();
            while iter.valid() {
                let k = iter
                    .key()
                    .ok_or_else(|| RoochError::UnexpectedError("iter key none".to_owned()))?;
                if k.len() == 64 {
                    let node = k[32..64].to_vec();
                    nodes.insert(node.clone());
                    // tx_order is stored in the low 64 bits (big-endian) of the first 32 bytes
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(&k[24..32]);
                    let order = u64::from_be_bytes(buf);
                    order_map
                        .entry(node)
                        .and_modify(|(cnt, min_o, max_o): &mut (usize, u64, u64)| {
                            *cnt += 1;
                            *min_o = (*min_o).min(order);
                            *max_o = (*max_o).max(order);
                        })
                        .or_insert((1usize, order, order));
                } else {
                    weird += 1;
                }
                iter.next();
            }
            (nodes, weird, order_map)
        };

        let missing_raw: Vec<_> = state32.difference(&refc32).cloned().collect();
        let missing_after_stale: Vec<_> = missing_raw
            .iter()
            .cloned()
            .filter(|k| !stale_nodes.contains(k))
            .collect();
        let extra: Vec<_> = refc32.difference(&state32).cloned().collect();
        // Classify extra_rc by whether they appear in stale_index (with tx_order stats)
        let mut extra_in_stale = Vec::new();
        let mut extra_not_in_stale = Vec::new();
        for k in extra.iter() {
            if let Some((cnt, min_o, max_o)) = stale_order_map.get(k) {
                extra_in_stale.push((k.clone(), *cnt, *min_o, *max_o));
            } else {
                extra_not_in_stale.push(k.clone());
            }
        }

        let mut out = String::new();
        use std::fmt::Write as _;
        writeln!(
            out,
            "state_node (32B keys): {} (weird len: {})",
            state32.len(),
            state_weird
        )
        .ok();
        writeln!(
            out,
            "node_refcount (32B keys): {} (weird len: {})",
            refc32.len(),
            refc_weird
        )
        .ok();
        writeln!(
            out,
            "smt_stale entries (parsed nodes): {} (weird len: {})",
            stale_nodes.len(),
            stale_weird
        )
        .ok();
        writeln!(
            out,
            "state_without_refcount (raw): {} | after_excluding_stale: {}",
            missing_raw.len(),
            missing_after_stale.len()
        )
        .ok();
        for k in missing_after_stale.iter().take(self.sample) {
            writeln!(out, "  missing_rc {}", hex::encode(k)).ok();
        }
        writeln!(out, "refcount_without_state: {}", extra.len()).ok();
        writeln!(
            out,
            "  in_stale: {} (showing up to {})",
            extra_in_stale.len(),
            self.sample
        )
        .ok();
        for (k, cnt, min_o, max_o) in extra_in_stale.iter().take(self.sample) {
            writeln!(
                out,
                "    extra_rc {} | stale_entries={} tx_order_range=[{},{}]",
                hex::encode(k),
                cnt,
                min_o,
                max_o
            )
            .ok();
        }
        writeln!(
            out,
            "  not_in_stale: {} (showing up to {})",
            extra_not_in_stale.len(),
            self.sample
        )
        .ok();
        for k in extra_not_in_stale.iter().take(self.sample) {
            writeln!(out, "    extra_rc {}", hex::encode(k)).ok();
        }

        Ok(out)
    }
}

fn collect_keys(
    db: &rocksdb::DB,
    cf_name: &str,
    expected_len: usize,
) -> RoochResult<(HashSet<Vec<u8>>, usize)> {
    let cf = db
        .cf_handle(cf_name)
        .ok_or_else(|| RoochError::UnexpectedError(format!("missing {} cf", cf_name)))?;
    let mut iter = db.raw_iterator_cf(cf);
    iter.seek_to_first();
    let mut set = HashSet::new();
    let mut weird = 0usize;
    while iter.valid() {
        let k = iter
            .key()
            .ok_or_else(|| RoochError::UnexpectedError("iter key none".to_owned()))?;
        if k.len() == expected_len {
            set.insert(k.to_vec());
        } else {
            weird += 1;
        }
        iter.next();
    }
    Ok((set, weird))
}
