// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::address::RoochAddress;
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::LedgerTransaction;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

const DEFAULT_MAX_SEGMENT_SIZE: usize = 4 * 1024 * 1024;

/// Unpack human-readable LedgerTransaction List to segments.
#[derive(Debug, Parser)]
pub struct PackCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "batch-path")]
    pub batch_path: PathBuf,
    #[clap(long = "chunk-id")]
    pub chunk_id: u128,
    #[clap(long)]
    pub sequencer_account: Option<String>,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl PackCommand {
    pub fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build()?;
        let sequencer_account = if self.sequencer_account.is_none() {
            let active_address_opt = context.client_config.active_address;
            if active_address_opt.is_none() {
                return Err(RoochError::ActiveAddressDoesNotExistError);
            }
            active_address_opt.unwrap()
        } else {
            RoochAddress::from_str(self.sequencer_account.clone().unwrap().as_str()).map_err(
                |e| {
                    RoochError::CommandArgumentError(format!(
                        "Invalid sequencer account address: {}",
                        e
                    ))
                },
            )?
        };
        let sequencer_keypair = context
            .keystore
            .get_key_pair(&sequencer_account, None)
            .map_err(|e| RoochError::SequencerKeyPairDoesNotExistError(e.to_string()))?;

        let mut reader = BufReader::new(File::open(self.batch_path)?);
        let mut tx_list = Vec::new();
        for line in reader.by_ref().lines() {
            let line = line?;
            let tx: LedgerTransaction = serde_json::from_str(&line)?;
            tx_list.push(tx);
        }
        let tx_order_start = tx_list.first().unwrap().sequence_info.tx_order;
        let tx_order_end = tx_list.last().unwrap().sequence_info.tx_order;

        let batch = DABatch::new(
            self.chunk_id,
            tx_order_start,
            tx_order_end,
            &tx_list,
            sequencer_keypair,
        );
        batch.verify(true)?;

        let segments = ChunkV0::from(batch).to_segments(DEFAULT_MAX_SEGMENT_SIZE);
        for segment in segments.iter() {
            let segment_path = self.segment_dir.join(segment.get_id().to_string());
            let mut writer = File::create(segment_path)?;
            writer.write_all(&segment.to_bytes())?;
            writer.flush()?;
        }

        Ok(())
    }
}
