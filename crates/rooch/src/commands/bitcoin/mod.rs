// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::{consensus::Encodable, Psbt, Transaction, Txid};
use broadcast_tx::BroadcastTx;
use build_tx::BuildTx;
use clap::{Parser, Subcommand};
use rooch_types::error::RoochResult;
use serde::{Deserialize, Serialize};
use sign_tx::SignTx;
use std::{env, fs::File, io::Write, path::PathBuf};
use transfer::Transfer;

pub mod broadcast_tx;
pub mod build_tx;
pub mod sign_tx;
pub mod transaction_builder;
pub mod transfer;
pub mod utxo_selector;

#[derive(Debug, Parser)]
pub struct Bitcoin {
    #[clap(subcommand)]
    cmd: BitcoinCommands,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum BitcoinCommands {
    BuildTx(BuildTx),
    SignTx(SignTx),
    BroadcastTx(BroadcastTx),
    Transfer(Transfer),
}

#[async_trait]
impl CommandAction<String> for Bitcoin {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            BitcoinCommands::BuildTx(build_tx) => build_tx.execute_serialized().await,
            BitcoinCommands::SignTx(sign_tx) => sign_tx.execute_serialized().await,
            BitcoinCommands::BroadcastTx(broadcast_tx) => broadcast_tx.execute_serialized().await,
            BitcoinCommands::Transfer(transfer) => transfer.execute_serialized().await,
        }
    }
}

pub(crate) enum FileOutputData {
    Psbt(Psbt),
    Tx(Transaction),
}

impl FileOutputData {
    pub fn txid(&self) -> Txid {
        match self {
            FileOutputData::Psbt(psbt) => psbt.unsigned_tx.compute_txid(),
            FileOutputData::Tx(tx) => tx.compute_txid(),
        }
    }

    pub fn file_suffix(&self) -> &str {
        match self {
            FileOutputData::Psbt(_) => "psbt",
            FileOutputData::Tx(_) => "tx",
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            FileOutputData::Psbt(psbt) => psbt.serialize(),
            FileOutputData::Tx(tx) => {
                let mut buf = Vec::new();
                tx.consensus_encode(&mut buf)
                    .expect("encode tx should success");
                buf
            }
        }
    }

    pub fn default_output_file_path(&self) -> Result<PathBuf> {
        let temp_dir = env::temp_dir();
        let tx_hash = self.txid();
        let file_name = format!("{}.{}", hex::encode(&tx_hash[..8]), self.file_suffix());
        Ok(temp_dir.join(file_name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileOutput {
    pub content: String,
    pub output_type: String,
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
            output_type: data.file_suffix().to_string(),
            path: path.to_string_lossy().to_string(),
        })
    }
}
