// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_types::h256::H256;
use rooch_types::transaction::{
    rooch::PartiallySignedRoochTransaction, RoochTransaction, RoochTransactionData,
};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Write, path::PathBuf};

pub mod build;
pub mod get_transactions_by_hash;
pub mod get_transactions_by_order;
pub mod query;
pub mod sign;
pub mod submit;

pub(crate) enum FileOutputData {
    RoochTransactionData(RoochTransactionData),
    SignedRoochTransaction(RoochTransaction),
    PartiallySignedRoochTransaction(PartiallySignedRoochTransaction),
}

impl FileOutputData {
    pub fn tx_hash(&self) -> H256 {
        match self {
            FileOutputData::RoochTransactionData(data) => data.tx_hash(),
            FileOutputData::SignedRoochTransaction(data) => data.data.tx_hash(),
            FileOutputData::PartiallySignedRoochTransaction(data) => data.data.tx_hash(),
        }
    }

    pub fn file_suffix(&self) -> &str {
        match self {
            FileOutputData::RoochTransactionData(_) => "rtd",
            FileOutputData::SignedRoochTransaction(_) => "srt",
            FileOutputData::PartiallySignedRoochTransaction(_) => "psrt",
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            FileOutputData::RoochTransactionData(data) => data.encode(),
            FileOutputData::SignedRoochTransaction(data) => data.encode(),
            FileOutputData::PartiallySignedRoochTransaction(data) => data.encode(),
        }
    }

    pub fn default_output_file_path(&self) -> Result<PathBuf> {
        let temp_dir = env::temp_dir();
        let tx_hash = self.tx_hash();
        let file_name = format!("{}.{}", hex::encode(&tx_hash[..8]), self.file_suffix());
        Ok(temp_dir.join(file_name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileOutput {
    pub content: String,
    pub path: String,
}

impl FileOutput {
    pub fn write_to_file(data: FileOutputData, output_path: Option<String>) -> Result<Self> {
        let path = match output_path {
            Some(path) => PathBuf::from(path),
            None => data.default_output_file_path()?,
        };
        let mut file = File::create(&path)?;
        // we write the hex encoded data to the file
        // not the binary data, for better readability
        let hex = hex::encode(data.encode());
        file.write_all(hex.as_bytes())?;
        Ok(FileOutput {
            content: hex,
            path: path.to_string_lossy().to_string(),
        })
    }
}
