// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use rooch_benchmarks::helper::profiled;
use rooch_types::transaction::L1Block;

pub fn l1_block_encode_size_benchmark(c: &mut Criterion) {
    let l1_block = L1Block::default();
    c.bench_function("l1_block_encode_siz", |b| {
        b.iter(|| {
            let _ = l1_block.encode().len() as u64;
        })
    });
}

pub fn l1_block_serialized_size_benchmark(c: &mut Criterion) {
    let l1_block = L1Block::default();
    c.bench_function("l1_block_serialized_size", |b| {
        b.iter(|| {
            let _ = bcs::serialized_size(&l1_block).unwrap() as u64;
        })
    });
}

criterion_group! {
    name = l1_block_encode_size_bench;
    config = profiled(None);
    targets = l1_block_encode_size_benchmark
}

criterion_group! {
    name = l1_block_serialized_size_bench;
    config = profiled(None);
    targets = l1_block_serialized_size_benchmark
}

criterion_main!(l1_block_encode_size_bench, l1_block_serialized_size_bench);
