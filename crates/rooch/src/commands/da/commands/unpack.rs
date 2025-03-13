// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{collect_chunk, collect_chunks, get_tx_list_from_chunk};
use clap::Parser;
use rooch_types::error::RoochResult;
use std::collections::{HashMap, HashSet};

use crate::utils::TxSizeHist;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Unpack batches to human-readable LedgerTransaction List from segments directory.
#[derive(Debug, Parser)]
pub struct UnpackCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "batch-dir")]
    pub batch_dir: PathBuf,
    #[clap(
        long = "chunk-id",
        help = "Only unpack the specified chunk_id, otherwise unpack all chunks"
    )]
    pub chunk_id: Option<u128>,
    #[clap(long = "stats-only", help = "Only print L2Tx size stats, no unpacking")]
    pub stats_only: bool,
    #[clap(long = "force", help = "Force unpacking, even if the batch has issues")]
    pub force: bool,
}

impl UnpackCommand {
    pub fn execute(self) -> RoochResult<()> {
        let mut unpacker = UnpackInner {
            unpacked: HashSet::new(),
            chunks: Default::default(),
            segment_dir: self.segment_dir,
            batch_dir: self.batch_dir,
            stats_only: self.stats_only,
        };
        unpacker.unpack(self.force, self.chunk_id)?;

        Ok(())
    }
}

struct UnpackInner {
    unpacked: HashSet<u128>,
    chunks: HashMap<u128, Vec<u64>>,
    segment_dir: PathBuf,
    batch_dir: PathBuf,
    stats_only: bool,
}

impl UnpackInner {
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

    fn collect_chunks(&mut self, unpack_chunk_id_opt: Option<u128>) -> anyhow::Result<()> {
        let chunks = if let Some(chunk_id) = unpack_chunk_id_opt {
            let segment_numbers = collect_chunk(self.segment_dir.clone(), chunk_id)?;
            let mut chunks = HashMap::new();
            chunks.insert(chunk_id, segment_numbers);
            chunks
        } else {
            let (chunks, _min_chunk_id, _max_chunk_id) = collect_chunks(self.segment_dir.clone())?;
            chunks
        };

        self.chunks = chunks;

        Ok(())
    }

    // unpack batches from segment_dir to batch_dir.
    // warn: ChunkV0 only in present
    fn unpack(&mut self, force: bool, unpack_chunk_id_opt: Option<u128>) -> anyhow::Result<()> {
        const TOP_N: usize = 20;

        self.collect_unpacked()?;
        self.collect_chunks(unpack_chunk_id_opt)?;

        let mut new_unpacked = HashSet::new();

        let mut l2tx_hist = TxSizeHist::new("L2Tx".to_string(), TOP_N, None, None)?;

        for (chunk_id, segment_numbers) in &self.chunks {
            if self.unpacked.contains(chunk_id) {
                // For ChunkV0, chunk_id is block_number
                continue;
            }

            let tx_list = get_tx_list_from_chunk(
                self.segment_dir.clone(),
                *chunk_id,
                segment_numbers.clone(),
                !force,
            )?;

            for tx in &tx_list {
                let tx_order = tx.sequence_info.tx_order;
                if let rooch_types::transaction::LedgerTxData::L2Tx(tx) = &tx.data {
                    let tx_size = tx.tx_size();
                    l2tx_hist.record(tx_order, tx_size)?;
                }
            }

            if self.stats_only {
                continue;
            }
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

        l2tx_hist.print();

        Ok(())
    }
}
