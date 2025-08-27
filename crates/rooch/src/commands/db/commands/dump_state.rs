// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db;
use anyhow::anyhow;
use clap::Parser;
use metrics::RegistryService;
use moveos_store::state_store::statedb::STATEDB_DUMP_BATCH_SIZE;
use moveos_types::h256::H256;
use moveos_types::state::{FieldKey, ObjectState};
use raw_store::{CodecKVStore, SchemaStore};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::{InMemoryNodeStore, NodeReader, SMTree, UpdateSet, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;
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

        // 1) 先 dump 顶层 KV
        let top_kvs = smt
            .dump(state_root)
            .map_err(|e| anyhow!("Failed to read state data: {}", e))?;
        let total_top_kv = top_kvs.len();

        // 2) 使用内存树重建顶层节点，作为初始节点集
        let registry = RegistryService::default().default_registry();
        let mem_store = InMemoryNodeStore::default();
        let mem_tree: SMTree<FieldKey, ObjectState, InMemoryNodeStore> =
            SMTree::new(mem_store.clone(), &registry);

        let mut updates = UpdateSet::new();
        for (k, v) in top_kvs.iter().cloned() {
            updates.put(k, v);
        }

        let empty_root: H256 = (*SPARSE_MERKLE_PLACEHOLDER_HASH).into();
        let top_change = mem_tree
            .puts(empty_root, updates)
            .map_err(|e| anyhow!("Rebuild top tree in-memory failed: {}", e))?;

        if top_change.state_root != state_root {
            return Err(anyhow!(
                "Top tree rebuilt root mismatch. expected: {:#x}, rebuilt: {:#x}",
                state_root,
                top_change.state_root
            )
                .into());
        }

        // 3) 广度优先遍历所有含字段的对象，递归 dump 子树并重建节点
        let mut combined_nodes: BTreeMap<H256, Vec<u8>> = top_change.nodes.clone();
        let mut visited_roots: BTreeSet<H256> = BTreeSet::new();
        visited_roots.insert(state_root);

        let mut total_objects: usize = top_kvs.len();
        let mut total_roots: usize = 1;

        let mut queue: VecDeque<H256> = VecDeque::new();

        // 将所有有字段的对象的 state_root 入队
        for (_k, v) in top_kvs.iter() {
            if v.metadata.has_fields() {
                let child_root = v.state_root();
                if visited_roots.insert(child_root) {
                    queue.push_back(child_root);
                    total_roots += 1;
                }
            }
        }

        while let Some(root) = queue.pop_front() {
            // 3.1 dump 子树 kv
            let kvs = smt
                .dump(root)
                .map_err(|e| anyhow!("Failed to dump subtree root {:#x}: {}", root, e))?;
            total_objects += kvs.len();

            // 3.2 重建该子树节点并校验
            let mut sub_updates = UpdateSet::new();
            for (k, v) in kvs.iter().cloned() {
                sub_updates.put(k, v);
            }
            let sub_change = mem_tree
                .puts(empty_root, sub_updates)
                .map_err(|e| anyhow!("Rebuild subtree in-memory failed for root {:#x}: {}", root, e))?;

            if sub_change.state_root != root {
                return Err(anyhow!(
                    "Subtree rebuilt root mismatch. expected: {:#x}, rebuilt: {:#x}",
                    root,
                    sub_change.state_root
                )
                    .into());
            }

            // 3.3 合并节点
            for (h, bytes) in sub_change.nodes {
                combined_nodes.entry(h).or_insert(bytes);
            }

            // 3.4 继续向下发现更多子树
            for (_k, v) in kvs.iter() {
                if v.metadata.has_fields() {
                    let next_root = v.state_root();
                    if visited_roots.insert(next_root) {
                        queue.push_back(next_root);
                        total_roots += 1;
                    }
                }
            }
        }

        // 4) 输出：模式仍采用 nodes，以包含所有树的全部节点
        let head = format!(
            "# mode=nodes\n# root={:#x}\n# roots_total={}\n# nodes_total={}\n# objects_total={}\n",
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
