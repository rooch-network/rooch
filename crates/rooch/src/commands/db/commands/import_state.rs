// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db;
use anyhow::{anyhow, Result};
use clap::Parser;
use metrics::RegistryService;
use moveos_types::h256::H256;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::FieldKey;
use moveos_types::state::ObjectState;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::{NodeReader, SMTree, UpdateSet};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;
use raw_store::{CodecKVStore, SchemaStore};

#[derive(Debug, Parser)]
pub struct ImportStateCommand {
    #[clap(long, short = 'i')]
    input_file: PathBuf,

    #[clap(long)]
    expected_state_root: Option<String>,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n')]
    pub chain_id: Option<RoochChainID>,
}

impl ImportStateCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        // rooch_db.rooch_store.state_store.save_state_change_set();

        // 从头部读取 root，如果 CLI 也提供了 expected_state_root，则做一致性校验
        let mut file = BufReader::new(
            File::open(&self.input_file)
                .map_err(|e| anyhow!("Failed to open input file: {}", e))?,
        );

        // 读取前几行头部
        let mut header_lines = Vec::new();
        for _ in 0..4 {
            let mut line = String::new();
            let n = file
                .read_line(&mut line)
                .map_err(|e| anyhow!("Failed to read header: {}", e))?;
            if n == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with('#') {
                header_lines.push(line);
            } else {
                // 遇到非注释，回退缓冲逻辑：把这一行留待后续按节点行解析
                header_lines.push(String::new()); // 占位，保持行为一致
                                                  // 将文件指针复位到行首的简易方式：重新打开文件，后续整体按行读取
                                                  // （为简单与可移植，这里不做复杂的 seek 处理）
                drop(file);
                file = BufReader::new(
                    File::open(&self.input_file)
                        .map_err(|e| anyhow!("Failed to reopen input file: {}", e))?,
                );
                break;
            }
        }

        // 再整体读取行（包括头部）
        let lines = BufReader::new(
            File::open(&self.input_file)
                .map_err(|e| anyhow!("Failed to open input file: {}", e))?,
        )
        .lines();

        // 解析头部信息
        let mut mode_nodes = false;
        let mut header_root: Option<H256> = None;
        let mut header_nodes_count: Option<usize> = None;

        for l in &header_lines {
            if l.starts_with("# mode=") {
                let v = l.trim().strip_prefix("# mode=").unwrap_or("").trim();
                mode_nodes = v.eq_ignore_ascii_case("nodes");
            }
            if l.starts_with("# root=") {
                let v = l.trim().strip_prefix("# root=").unwrap_or("").trim();
                if !v.is_empty() {
                    let root = H256::from_str(v)
                        .map_err(|e| anyhow!("Invalid root in header '{}': {}", v, e))?;
                    header_root = Some(root);
                }
            }
            if l.starts_with("# nodes=") {
                let v = l.trim().strip_prefix("# nodes=").unwrap_or("").trim();
                if !v.is_empty() {
                    let n = v
                        .parse::<usize>()
                        .map_err(|e| anyhow!("Invalid nodes count in header '{}': {}", v, e))?;
                    header_nodes_count = Some(n);
                }
            }
        }

        if !mode_nodes {
            return Err(
                anyhow!("Input file must be node-mode dump (missing '# mode=nodes')").into(),
            );
        }
        let header_root = header_root.ok_or_else(|| anyhow!("Missing '# root=...' header"))?;

        if let Some(expect_hex) = self.expected_state_root.as_ref() {
            let expect = H256::from_str(expect_hex)
                .map_err(|e| anyhow!("Invalid expected state root hash: {}", e))?;
            if expect != header_root {
                return Err(anyhow!(
                    "Expected root {} mismatches file root {}",
                    expect,
                    header_root
                )
                .into());
            }
        }

        // 解析正文节点 <hash_hex>:0x<bytes_hex>
        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();
        let mut total_nodes = 0usize;

        for line in lines {
            let line = line.map_err(|e| anyhow!("Failed to read line from file: {}", e))?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid node line, expected 'hash:0xHEX': {}", line).into());
            }
            let hash_hex = parts[0].trim();
            let bytes_hex = parts[1]
                .trim()
                .strip_prefix("0x")
                .unwrap_or(parts[1].trim());

            let hash = H256::from_str(hash_hex)
                .map_err(|e| anyhow!("Invalid node hash '{}': {}", hash_hex, e))?;
            let bytes = hex::decode(bytes_hex)
                .map_err(|e| anyhow!("Invalid node hex for {}: {}", hash_hex, e))?;
            nodes.insert(hash, bytes);
            total_nodes += 1;
        }

        if let Some(expect_nodes) = header_nodes_count {
            if expect_nodes != total_nodes {
                return Err(anyhow!(
                    "Nodes count mismatch. header={}, actual={}",
                    expect_nodes,
                    total_nodes
                )
                .into());
            }
        }

        // 批量写节点
        let state_store = rooch_db.moveos_store.get_state_store();
        state_store
            .update_nodes(nodes)
            .map_err(|e| anyhow!("write_nodes failed: {}", e))?;

        let v = rooch_db.moveos_store.node_store.get_store().store();
        v.db().expect("db 1111").flush_all()?;

        // 确认根节点已经存在
        let has_root = state_store
            .node_store
            .get(&header_root)
            .map_err(|e| anyhow!("node_store.get(root) failed: {}", e))?
            .is_some();
        if !has_root {
            return Err(anyhow!(
                "Root node {:#x} not present after import_nodes. Aborting.",
                header_root
            )
            .into());
        }

        // 更新 StartupInfo.state_root（保留原 size）
        let config_store = &rooch_db.moveos_store.config_store;
        let mut startup_info = config_store
            .get_startup_info()
            .map_err(|e| anyhow!("Failed to get startup info: {}", e))?
            .unwrap_or_else(|| StartupInfo::new(header_root, 0));
        let keep_size = startup_info.get_size();
        startup_info.update_state_root(header_root, keep_size);
        config_store
            .save_startup_info(startup_info)
            .map_err(|e| anyhow!("Failed to save startup info: {}", e))?;

        info!(
            "Imported {} nodes, set latest root to {:#x}",
            total_nodes, header_root
        );
        Ok(format!("Imported {} nodes", total_nodes))
    }
}

fn parse_field_key(key_str: &str) -> Result<FieldKey> {
    let key_str = key_str.strip_prefix("0x").unwrap_or(key_str);

    let bytes = hex::decode(key_str)
        .map_err(|e| anyhow!("Failed to decode hex string to FieldKey: {}", e))?;

    if bytes.len() != FieldKey::LENGTH {
        return Err(anyhow!(
            "Incorrect FieldKey byte length: expected {}, got {}",
            FieldKey::LENGTH,
            bytes.len()
        ));
    }

    let mut array = [0u8; FieldKey::LENGTH];
    array.copy_from_slice(&bytes);

    Ok(FieldKey::new(array))
}

fn parse_object_state(value_str: &str) -> Result<ObjectState> {
    let value_str = value_str.strip_prefix("0x").unwrap_or(value_str);

    let bytes = hex::decode(value_str)
        .map_err(|e| anyhow!("Failed to decode hex string to ObjectState: {}", e))?;

    ObjectState::from_bytes(&bytes)
        .map_err(|e| anyhow!("Failed to create ObjectState from byte array: {}", e))
}
