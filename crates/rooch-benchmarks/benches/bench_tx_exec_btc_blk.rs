// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main};

use rooch_benchmarks::config::configure_criterion;
use rooch_benchmarks::tx_exec::tx_exec_benchmark;

criterion_group! {
    name = tx_exec_btc_blk_bench;
    config = configure_criterion(None).sample_size(10); // block after 800,000 always need seconds/block
    targets = tx_exec_benchmark
}

criterion_main!(tx_exec_btc_blk_bench);
