// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{collect_chunks, get_tx_list_from_chunk};
use clap::Parser;
use rooch_types::error::RoochResult;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;

/// Unpack batches to human-readable LedgerTransaction List from segments directory.
#[derive(Debug, Parser)]
pub struct UnpackCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "batch-dir")]
    pub batch_dir: PathBuf,
    #[clap(
        long = "verify-order",
        help = "Verify the order of transactions for all batches"
    )]
    pub verify_order: bool,
}

impl UnpackCommand {
    pub fn execute(self) -> RoochResult<()> {
        let mut unpacker = UnpackInner {
            unpacked: HashSet::new(),
            chunks: Default::default(),
            segment_dir: self.segment_dir,
            batch_dir: self.batch_dir,
        };
        unpacker.unpack()?;
        if self.verify_order {
            unpacker.verify_order()?;
        }

        Ok(())
    }
}

struct UnpackInner {
    unpacked: HashSet<u128>,
    chunks: HashMap<u128, Vec<u64>>,
    segment_dir: PathBuf,
    batch_dir: PathBuf,
}

impl UnpackInner {
    fn verify_order(&self) -> anyhow::Result<()> {
        let mut max_block_number = 0;
        let mut last_tx_order = 0;
        // start from block_number 0,
        // read from batch_dir/<block_number> and verify the order of transactions, until no file found.
        loop {
            let batch_file_path = self.batch_dir.join(max_block_number.to_string());
            if !batch_file_path.exists() {
                break;
            }

            let file = fs::File::open(batch_file_path)?;
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let tx: rooch_types::transaction::LedgerTransaction = serde_json::from_str(&line)?;
                let tx_order = tx.sequence_info.tx_order;
                if tx_order != last_tx_order + 1 {
                    return Err(anyhow::anyhow!(
                        "Transaction order is not strictly incremental for block {}: last_tx_order: {}, tx_order: {}",
                        max_block_number, last_tx_order, tx_order
                    ));
                }
                last_tx_order = tx_order;
            }

            if max_block_number % 1000 == 0 && max_block_number > 0 {
                println!("Verified block: {}", max_block_number);
            }

            max_block_number += 1;
        }
        println!(
            "All transactions are strictly incremental for blocks: [0, {}). last_tx_order: {}",
            max_block_number, last_tx_order
        );
        Ok(())
    }

    // batch_dir is a directory that stores all the unpacked batches.
    // each batch is stored in a file named by the block number (each batch maps to a block).
    // we collect all the block numbers to avoid unpacking the same batch multiple times.
    fn collect_unpacked(&mut self) -> anyhow::Result<()> {
        let batch_dir = self.batch_dir.clone();
        if !batch_dir.exists() {
            fs::create_dir_all(&batch_dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(batch_dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(batch_id) = path
                        .file_name()
                        .and_then(|s| s.to_str()?.parse::<u128>().ok())
                    {
                        self.unpacked.insert(batch_id);
                    } else {
                        eprintln!("Failed to parse batch_id from path: {:?}", path);
                    }
                }
            } else {
                eprintln!("Failed to read entry: {:?}", entry);
            }
        }
        Ok(())
    }

    fn collect_chunks(&mut self) -> anyhow::Result<()> {
        let chunks = collect_chunks(self.segment_dir.clone())?;
        self.chunks = chunks;
        Ok(())
    }

    // unpack batches from segment_dir to batch_dir.
    // warn: ChunkV0 only in present
    fn unpack(&mut self) -> anyhow::Result<()> {
        self.collect_unpacked()?;
        self.collect_chunks()?;

        let mut new_unpacked = HashSet::new();

        for (chunk_id, segment_numbers) in &self.chunks {
            if self.unpacked.contains(chunk_id) {
                // For ChunkV0, chunk_id is block_number
                continue;
            }

            let tx_list = get_tx_list_from_chunk(
                self.segment_dir.clone(),
                *chunk_id,
                segment_numbers.clone(),
            )?;

            // write LedgerTx in batch to file, each line is a tx in json
            let batch_file_path = self.batch_dir.join(chunk_id.to_string());
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(batch_file_path)?;
            let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().unwrap());

            for tx in tx_list {
                let tx_json = serde_json::to_string(&tx)?;
                writeln!(writer, "{}", tx_json).expect("Unable to write line");
            }
            writer.flush().expect("Unable to flush writer");
            file.sync_data().expect("Unable to sync file");

            new_unpacked.insert(*chunk_id);
        }

        println!("Unpacked batches(block_number): {:?}", new_unpacked);
        Ok(())
    }
}
