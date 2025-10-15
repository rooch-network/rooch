// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use anyhow::anyhow;
use clap::Parser;
use metrics::RegistryService;
use moveos_store::state_store::statedb::STATEDB_DUMP_BATCH_SIZE;
use moveos_types::h256::H256;
use moveos_types::state::{FieldKey, ObjectState};
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::{InMemoryNodeStore, NodeReader, SMTree, UpdateSet, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

/// dump state DB from RocksDB in disk
#[derive(Debug, Parser)]
pub struct DumpStateCommand {
    #[clap(long, short = 'o')]
    output_file: PathBuf,

    #[clap(long)]
    state_root: String,

    #[clap(long, default_value_t = STATEDB_DUMP_BATCH_SIZE)]
    batch_size: usize,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n')]
    pub chain_id: Option<RoochChainID>,
}

impl DumpStateCommand {
    fn dump_root_recursive<NR: NodeReader>(
        smt: &SMTree<FieldKey, ObjectState, NR>,
        mem_tree: &SMTree<FieldKey, ObjectState, InMemoryNodeStore>,
        empty_root: H256,
        root: H256,
        visited_roots: &mut BTreeSet<H256>,
        combined_nodes: &mut BTreeMap<H256, Vec<u8>>,
        total_objects: &mut usize,
        total_roots: &mut usize,
    ) -> anyhow::Result<()> {
        // 0) Avoid the visited roots
        if !visited_roots.insert(root) {
            return Ok(());
        }
        *total_roots += 1;

        // 1) Dump the kv pairs under this root
        let kvs = smt
            .dump(root)
            .map_err(|e| anyhow!("Failed to dump subtree root {:#x}: {}", root, e))?;
        *total_objects += kvs.len();

        // 2) Rebuild the subtree nodes and verify
        let mut updates = UpdateSet::new();
        for (k, v) in kvs.iter().cloned() {
            updates.put(k, v);
        }
        let change = mem_tree.puts(empty_root, updates).map_err(|e| {
            anyhow!(
                "Rebuild subtree in-memory failed for root {:#x}: {}",
                root,
                e
            )
        })?;

        if change.state_root != root {
            return Err(anyhow!(
                "Subtree rebuilt root mismatch. expected: {:#x}, rebuilt: {:#x}",
                root,
                change.state_root
            ));
        }

        // 3) Merge nodes
        for (h, bytes) in change.nodes {
            combined_nodes.entry(h).or_insert(bytes);
        }

        // 4) Recursively go into subtrees
        for (_k, v) in kvs.iter() {
            if v.metadata.has_fields() {
                let child_root = v.state_root();
                Self::dump_root_recursive(
                    smt,
                    mem_tree,
                    empty_root,
                    child_root,
                    visited_roots,
                    combined_nodes,
                    total_objects,
                    total_roots,
                )?;
            }
        }

        Ok(())
    }

    pub async fn execute(self) -> RoochResult<String> {
        let state_root = H256::from_str(&self.state_root)
            .map_err(|e| anyhow!("Invalid state root hash: {}", e))?;

        let mut output_file = File::create(&self.output_file)
            .map_err(|e| anyhow!("Failed to create output file: {}", e))?;

        let (_root, rooch_db, _start_time) =
            open_rooch_db_readonly(self.base_data_dir.clone(), self.chain_id.clone());
        let state_store = rooch_db.moveos_store.get_state_store();
        let smt = &state_store.smt;

        // Prepare in-memory tree and auxiliary structures
        let registry = RegistryService::default().default_registry();
        let mem_store = InMemoryNodeStore::default();
        let mem_tree: SMTree<FieldKey, ObjectState, InMemoryNodeStore> =
            SMTree::new(mem_store.clone(), &registry);
        let empty_root: H256 = *SPARSE_MERKLE_PLACEHOLDER_HASH;

        let mut combined_nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();
        let mut visited_roots: BTreeSet<H256> = BTreeSet::new();
        let mut total_objects: usize = 0;
        let mut total_roots: usize = 0;

        // Recursively export the entire state tree starting from the top-level root
        Self::dump_root_recursive(
            smt,
            &mem_tree,
            empty_root,
            state_root,
            &mut visited_roots,
            &mut combined_nodes,
            &mut total_objects,
            &mut total_roots,
        )
        .map_err(|e| anyhow!("Dump recursive failed: {}", e))?;

        // Output: includes all nodes of all trees
        let head = format!(
            "# root={:#x}\n# roots_total={}\n# nodes_total={}\n# objects_total={}\n",
            state_root,
            total_roots,
            combined_nodes.len(),
            total_objects
        );
        output_file
            .write_all(head.as_bytes())
            .map_err(|e| anyhow!("Failed to write header: {}", e))?;

        for (h, b) in &combined_nodes {
            let line = format!("{:x}:0x{}\n", h, hex::encode(b));
            output_file
                .write_all(line.as_bytes())
                .map_err(|e| anyhow!("Failed to write node line: {}", e))?;
        }

        output_file
            .flush()
            .map_err(|e| anyhow!("Failed to flush file: {}", e))?;

        let result = format!(
            "Successfully exported ALL NODES to {:?}, roots={}, nodes_total={}, objects_total={}",
            self.output_file,
            total_roots,
            combined_nodes.len(),
            total_objects
        );
        info!("{}", result);
        Ok(result)
    }
}
