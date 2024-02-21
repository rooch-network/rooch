mod bench_tx_query;
mod bench_tx_write;

use bench_tx_query::transaction_query_benchmark;
use bench_tx_write::transaction_write_benchmark;
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

criterion_group! {
    name = rooch_tx_bench;
    config = Criterion::default().sample_size(200).measurement_time(Duration::from_secs(10));
    targets = transaction_write_benchmark, transaction_query_benchmark
}

criterion_main!(rooch_tx_bench);
