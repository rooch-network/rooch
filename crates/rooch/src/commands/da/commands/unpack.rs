// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{collect_chunks, get_tx_list_from_chunk};
use clap::Parser;
use rooch_types::error::RoochResult;
use std::collections::{BinaryHeap, HashMap, HashSet};

use std::cmp::Reverse;
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
        let (chunks, _min_chunk_id, _max_chunk_id) = collect_chunks(self.segment_dir.clone())?;
        self.chunks = chunks;

        if let Some(chunk_id) = unpack_chunk_id_opt {
            if !self.chunks.contains_key(&chunk_id) {
                return Err(anyhow::anyhow!(
                    "Chunk {} not found in segment_dir: {:?}",
                    chunk_id,
                    self.segment_dir
                ));
            } else {
                let segments = self.chunks.get(&chunk_id).unwrap().clone();
                self.chunks.clear();
                self.chunks.insert(chunk_id, segments);
            }
        }

        Ok(())
    }

    // unpack batches from segment_dir to batch_dir.
    // warn: ChunkV0 only in present
    fn unpack(&mut self, force: bool, unpack_chunk_id_opt: Option<u128>) -> anyhow::Result<()> {
        const TOP_N: usize = 20;

        self.collect_unpacked()?;
        self.collect_chunks(unpack_chunk_id_opt)?;

        let mut new_unpacked = HashSet::new();

        let mut l2tx_hist = TxStats {
            hist: hdrhistogram::Histogram::<u64>::new_with_bounds(1, 4_096_000, 3)?,
            tops: BinaryHeap::new(),
            top_n: TOP_N,
        };

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

struct TxStats {
    hist: hdrhistogram::Histogram<u64>,
    tops: BinaryHeap<Reverse<(u64, u64)>>, // (tx_size, tx_order) Use Reverse to keep the smallest element at the top
    top_n: usize,
}

impl TxStats {
    fn record(&mut self, tx_order: u64, tx_size: u64) -> anyhow::Result<()> {
        self.hist.record(tx_size)?;

        if self.tops.len() < self.top_n {
            // Add the new item directly if space is available
            self.tops.push(Reverse((tx_size, tx_order)));
        } else if let Some(&Reverse((smallest_size, _))) = self.tops.peek() {
            // Compare with the smallest item in the heap
            if tx_size > smallest_size {
                self.tops.pop(); // Remove the smallest
                self.tops.push(Reverse((tx_size, tx_order))); // Add the new larger item
            }
        }
        // Keep only top-N
        Ok(())
    }

    /// Returns the top N items, sorted by `tx_size` in descending order
    pub fn get_top(&self) -> Vec<(u64, u64)> {
        let mut sorted: Vec<_> = self.tops.iter().map(|&Reverse(x)| x).collect();
        sorted.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by tx_size in descending order
        sorted
    }

    fn print(&mut self) {
        let hist = &self.hist;

        let min_size = hist.min();
        let max_size = hist.max();
        let mean_size = hist.mean();

        println!("-----------------L2Tx Size Stats-----------------");
        println!(
            "Tx Size Percentiles distribution(count: {}): min={}, max={}, mean={:.2}, stdev={:.2}: ",
            hist.len(),
            min_size,
            max_size,
            mean_size,
            hist.stdev()
        );
        let percentiles = [
            1.00, 5.00, 10.00, 20.00, 30.00, 40.00, 50.00, 60.00, 70.00, 80.00, 90.00, 95.00,
            99.00, 99.50, 99.90, 99.95, 99.99,
        ];
        for &p in &percentiles {
            let v = hist.value_at_percentile(p);
            println!("| {:6.2}th=[{}]", p, v);
        }

        // each pair one line
        println!("-------------Top{} transactions--------------", self.top_n);
        let tops = self.get_top();
        for (tx_size, tx_order) in &tops {
            println!("tx_order: {}, tx_size: {}", tx_order, tx_size);
        }
    }
}
