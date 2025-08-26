// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db;
use anyhow::anyhow;
use clap::Parser;
use metrics::RegistryService;
use moveos_store::state_store::statedb::STATEDB_DUMP_BATCH_SIZE;
use moveos_types::h256::H256;
use moveos_types::state::{FieldKey, ObjectState};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::jellyfish_merkle::node_type::Node as JellyNode;
use smt::{InMemoryNodeStore, NodeReader, SMTree, UpdateSet, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::collections::{BTreeMap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

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
    pub async fn execute(self) -> RoochResult<String> {
        let state_root = H256::from_str(&self.state_root)
            .map_err(|e| anyhow!("Invalid state root hash: {}", e))?;

        let mut output_file = File::create(&self.output_file)
            .map_err(|e| anyhow!("Failed to create output file: {}", e))?;

        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        let state_store = rooch_db.moveos_store.get_state_store();
        let smt = &state_store.smt;

        // 1) 拿到快照 KV
        let state_kvs = smt
            .dump(state_root)
            .map_err(|e| anyhow!("Failed to read state data: {}", e))?;
        let total_kv = state_kvs.len();

        // 2) 在内存中用“空树根”重建整棵树，得到完整 nodes
        let registry = RegistryService::default().default_registry();
        let mem_store = InMemoryNodeStore::default();
        let mem_tree: SMTree<FieldKey, ObjectState, InMemoryNodeStore> =
            SMTree::new(mem_store.clone(), &registry);

        let mut updates = UpdateSet::new();
        for (k, v) in state_kvs.into_iter() {
            updates.put(k, v);
        }

        // 空树根（占位根）
        let empty_root: H256 = (*SPARSE_MERKLE_PLACEHOLDER_HASH).into();

        let change = mem_tree
            .puts(empty_root, updates)
            .map_err(|e| anyhow!("Rebuild tree in-memory failed: {}", e))?;

        // 3) 校验重建结果
        if change.state_root != state_root {
            return Err(anyhow!(
                "Rebuilt root mismatch. expected: {:#x}, rebuilt: {:#x}",
                state_root,
                change.state_root
            )
            .into());
        }

        // 4) 导出节点（含根与全树）
        let head = format!(
            "# mode=nodes\n# root={:#x}\n# nodes={}\n# size={}\n",
            state_root,
            change.nodes.len(),
            total_kv
        );
        output_file
            .write_all(head.as_bytes())
            .map_err(|e| anyhow!("Failed to write header: {}", e))?;

        for (h, b) in &change.nodes {
            let line = format!("{:x}:0x{}\n", h, hex::encode(b));
            output_file
                .write_all(line.as_bytes())
                .map_err(|e| anyhow!("Failed to write node line: {}", e))?;
        }

        output_file
            .flush()
            .map_err(|e| anyhow!("Failed to flush file: {}", e))?;

        let result = format!(
            "Successfully exported NODES to {:?}, total {} nodes (kv size={})",
            self.output_file,
            change.nodes.len(),
            total_kv
        );
        info!("{}", result);
        Ok(result)
    }
}
