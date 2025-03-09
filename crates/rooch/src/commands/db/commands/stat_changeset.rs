// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_inner_rocks;
use clap::Parser;
use rooch_store::STATE_CHANGE_SET_COLUMN_FAMILY_NAME;
use rooch_types::error::RoochResult;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Get changeset by order
#[derive(Debug, Parser)]
pub struct StatChangesetCommand {
    #[clap(long = "src", help = "source path to rocksdb")]
    pub src: String,
}

impl StatChangesetCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let cf_name = STATE_CHANGE_SET_COLUMN_FAMILY_NAME.to_string();
        let db = open_inner_rocks(&self.src, vec![cf_name.clone()], true)?;
        let source_cf = db.cf_handle(&cf_name).unwrap();
        let iter = db.iterator_cf(source_cf, rocksdb::IteratorMode::Start);

        const TOP_N: usize = 20;

        let mut hist = Stats {
            hist: hdrhistogram::Histogram::<u64>::new_with_bounds(1, 4_096_000, 3)
                .expect("Failed to create histogram"),
            tops: BinaryHeap::new(),
            top_n: TOP_N,
        };

        for result in iter {
            let (key, value) = result.expect("Failed to get key-value");
            let tx_order: u64 = bcs::from_bytes(&key)?;
            hist.record(tx_order, value.len() as u64)?;
        }

        hist.print();
        Ok(())
    }
}

struct Stats {
    hist: hdrhistogram::Histogram<u64>,
    tops: BinaryHeap<Reverse<(u64, u64)>>, // (size, tx_order) Use Reverse to keep the smallest element at the top
    top_n: usize,
}

impl Stats {
    fn record(&mut self, tx_order: u64, size: u64) -> anyhow::Result<()> {
        self.hist.record(size)?;

        if self.tops.len() < self.top_n {
            // Add the new item directly if space is available
            self.tops.push(Reverse((size, tx_order)));
        } else if let Some(&Reverse((smallest_size, _))) = self.tops.peek() {
            // Compare with the smallest item in the heap
            if size > smallest_size {
                self.tops.pop(); // Remove the smallest
                self.tops.push(Reverse((size, tx_order))); // Add the new larger item
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

        println!("-----------------Changset Size Stats-----------------");
        println!(
            "Changset Size Percentiles distribution(count: {}): min={}, max={}, mean={:.2}, stdev={:.2}: ",
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
            println!("tx_order: {}, changeset_size: {}", tx_order, tx_size);
        }
    }
}
