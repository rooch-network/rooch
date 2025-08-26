// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
use smt::{NodeReader, UpdateSet};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;
use rooch_types::rooch_network::RoochChainID;
use crate::utils::open_rooch_db;

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

        // 1) 必须提供目标根
        let expected_root_hex = self
            .expected_state_root
            .as_ref()
            .ok_or_else(|| anyhow!("--expected-state-root is required for nodes import"))?;
        let state_root = H256::from_str(expected_root_hex)
            .map_err(|e| anyhow!("Invalid expected state root hash: {}", e))?;

        // 2) 解析节点文件 <hash_hex>:0x<bytes_hex>
        let file = File::open(&self.input_file)
            .map_err(|e| anyhow!("Failed to open input file: {}", e))?;
        let reader = BufReader::new(file);

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();
        let mut total_nodes = 0usize;

        for line in reader.lines() {
            let line = line.map_err(|e| anyhow!("Failed to read line from file: {}", e))?;
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

        // 3) 批量写节点
        let state_store = rooch_db.moveos_store.get_state_store();
        state_store
            .update_nodes(nodes)
            .map_err(|e| anyhow!("write_nodes failed: {}", e))?;

        // 4) 确认根节点已经存在
        let has_root = state_store
            .node_store
            .get(&state_root)
            .map_err(|e| anyhow!("node_store.get(root) failed: {}", e))?
            .is_some();
        if !has_root {
            return Err(anyhow!(
                "Root node {:#x} not present after import_nodes. Aborting.",
                state_root
            )
            .into());
        }

        // 5) 更新 StartupInfo.state_root（保留原 size；如需更新，可扩展 CLI 参数）
        let config_store = &rooch_db.moveos_store.config_store;
        let mut startup_info = config_store
            .get_startup_info()
            .map_err(|e| anyhow!("Failed to get startup info: {}", e))?
            .unwrap_or_else(|| StartupInfo::new(state_root, 0));
        let keep_size = startup_info.get_size();
        startup_info.update_state_root(state_root, keep_size);
        config_store
            .save_startup_info(startup_info)
            .map_err(|e| anyhow!("Failed to save startup info: {}", e))?;

        info!(
            "Imported {} nodes, set latest root to {:#x}",
            total_nodes, state_root
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
