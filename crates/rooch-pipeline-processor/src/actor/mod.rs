// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use include_dir::{include_dir, Dir};
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod messages;
pub mod processor;

const STATIC_GENESIS_DIR: Dir = include_dir!("tx_anomalies");

pub fn load_tx_anomalies(genesis_namespace: String) -> anyhow::Result<Option<TxAnomalies>> {
    STATIC_GENESIS_DIR
        .get_file(genesis_namespace.as_str())
        .map(|f| {
            let tx_anomalies = TxAnomalies::decode(f.contents())?;
            Ok(tx_anomalies)
        })
        .transpose()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxAnomalies {
    pub genesis_namespace: String,
    pub dup_hash: HashMap<H256, Vec<u64>>,
    pub no_execution_info: HashMap<H256, u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accumulator_should_revert: Option<HashMap<u64, H256>>,
}

impl TxAnomalies {
    pub fn is_dup_hash(&self, hash: &H256) -> bool {
        self.dup_hash.contains_key(hash)
    }

    pub fn get_accumulator_should_revert(&self, order: u64) -> Option<H256> {
        self.accumulator_should_revert
            .as_ref()
            .and_then(|map| map.get(&order).cloned())
    }

    pub fn is_no_execution_info(&self, hash: &H256) -> bool {
        self.no_execution_info.contains_key(hash)
    }

    pub fn get_genesis_namespace(&self) -> String {
        self.genesis_namespace.clone()
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("TxAnomalies bcs::to_bytes should success")
    }

    pub fn load_from<P: AsRef<Path>>(genesis_file: P) -> anyhow::Result<Self> {
        let file_path = genesis_file.as_ref();
        let genesis_package_from_bcs = bcs::from_bytes(&std::fs::read(file_path)?);
        let genesis_package = match genesis_package_from_bcs {
            Ok(genesis_package) => genesis_package,
            Err(_) => {
                let genesis_package_from_json = serde_json::from_slice(&std::fs::read(file_path)?);
                match genesis_package_from_json {
                    Ok(genesis_package) => genesis_package,
                    Err(_) => return Err(anyhow::anyhow!("Failed to load genesis package")),
                }
            }
        };

        Ok(genesis_package)
    }

    pub fn save_to<P: AsRef<Path>>(&self, genesis_file: P) -> anyhow::Result<()> {
        let mut file = File::create(genesis_file)?;
        let contents = bcs::to_bytes(&self)?;
        file.write_all(&contents)?;
        file.sync_data()?;
        Ok(())
    }

    pub fn save_plain_text_to<P: AsRef<Path>>(&self, genesis_file: P) -> anyhow::Result<()> {
        let mut file = File::create(genesis_file)?;
        let contents = serde_json::to_string_pretty(&self)?;
        file.write_all(contents.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }
}
