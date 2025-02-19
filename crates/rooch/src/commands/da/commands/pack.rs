// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::da::commands::write_down_segments;
use crate::utils::get_sequencer_keypair;
use clap::Parser;
use rooch_types::error::RoochResult;
use rooch_types::transaction::LedgerTransaction;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

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
        let sequencer_keypair =
            get_sequencer_keypair(self.context_options, self.sequencer_account)?;

        let mut reader = BufReader::new(File::open(self.batch_path)?);
        let mut tx_list = Vec::new();
        for line in reader.by_ref().lines() {
            let line = line?;
            let tx: LedgerTransaction = serde_json::from_str(&line)?;
            tx_list.push(tx);
        }
        let tx_order_start = tx_list.first().unwrap().sequence_info.tx_order;
        let tx_order_end = tx_list.last().unwrap().sequence_info.tx_order;

        write_down_segments(
            self.chunk_id,
            tx_order_start,
            tx_order_end,
            &tx_list,
            &sequencer_keypair,
            self.segment_dir,
        )?;

        Ok(())
    }
}
