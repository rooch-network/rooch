// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main};

use rooch_benchmarks::helper::profiled;
use rooch_benchmarks::tx::tx_exec_benchmark;

criterion_group! {
    name = tx_exec_blk_bench;
    config = profiled(None).measurement_time(std::time::Duration::from_secs(20));
    targets = tx_exec_benchmark
}

criterion_main!(tx_exec_blk_bench);
