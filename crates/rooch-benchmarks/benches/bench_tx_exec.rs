// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main};

use rooch_benchmarks::config::configure_criterion;
use rooch_benchmarks::tx_exec::tx_exec_benchmark;

criterion_group! {
    name = tx_exec_bench;
    config = configure_criterion(None).measurement_time(std::time::Duration::from_secs(5));
    targets = tx_exec_benchmark
}

criterion_main!(tx_exec_bench);
