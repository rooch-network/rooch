// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use clap::Parser;
use metrics::RegistryService;
use moveos_types::h256::H256;
use moveos_types::state::FieldKey;
use moveos_types::state::ObjectState;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::error::RoochResult;
use smt::UpdateSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Parser)]
pub struct ImportStateCommand {
    #[clap(long, short = 'i')]
    input_file: PathBuf,

    #[clap(long)]
    expected_state_root: Option<String>,

    #[clap(flatten)]
    rooch_opt: RoochOpt,
}

impl ImportStateCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let registry_service = RegistryService::default();
        let rooch_db = RoochDB::init(
            self.rooch_opt.store_config(),
            &registry_service.default_registry(),
        )
            .map_err(|e| anyhow!("Failed to initialize RoochDB: {}", e))?;

        let root_meta = rooch_db
            .latest_root()
            .map_err(|e| anyhow!("Failed to fetch latest state root: {}", e))?;

        let mut state_root = root_meta.expect("").state_root.expect("");
        info!("Current state root: {:?}", state_root);

        let state_store = rooch_db.moveos_store.get_state_store();
        let smt = &state_store.smt;

        let file = File::open(&self.input_file)
            .map_err(|e| anyhow!("Failed to open input file: {}", e))?;
        let reader = BufReader::new(file);

        let mut batch = UpdateSet::new();
        let mut total_count = 0;

        for line in reader.lines() {
            let line = line.map_err(|e| anyhow!("Failed to read line from file: {}", e))?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid line format, expected 'key:value': {}", line).into());
            }

            let key_str = parts[0].trim();
            let value_str = parts[1].trim();

            let field_key = parse_field_key(key_str)
                .map_err(|e| anyhow!("Failed to parse FieldKey: {}, line: {}", e, line))?;
            let object_state = parse_object_state(value_str)
                .map_err(|e| anyhow!("Failed to parse ObjectState: {}, line: {}", e, line))?;

            batch.put(field_key, object_state);
            total_count += 1;
        }

        let change_set = smt
            .puts(state_root, batch)
            .map_err(|e| anyhow!("Failed to update state data: {}", e))?;

        let node_store = rooch_db.moveos_store.node_store;
        node_store
            .write_nodes(change_set.nodes)
            .expect("node_store.write_nodes failed.");

        state_root = change_set.state_root;
        info!("Imported {} records, new state root: {:?}", total_count, state_root);

        if let Some(expected_state_root) = self.expected_state_root {
            let expected_root = H256::from_str(&expected_state_root)
                .map_err(|e| anyhow!("Invalid expected state root hash: {}", e))?;

            if state_root != expected_root {
                return Err(anyhow!(
                    "State root validation failed! Expected: {:?}, actual: {:?}",
                    expected_root,
                    state_root
                )
                    .into());
            }

            info!("State root validation succeeded!");
        }

        let result = format!("Successfully imported {} records", total_count);
        Ok(result)
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
