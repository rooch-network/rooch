// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use rooch_benchmarks::config::{configure_criterion, BenchTxConfig};
use rooch_benchmarks::tx::create_l2_tx;
use rooch_framework_tests::binding_test;
use rooch_test_transaction_builder::TestTransactionBuilder;
use rooch_types::crypto::RoochKeyPair;

pub fn tx_validate_benchmark(c: &mut Criterion) {
    let config = BenchTxConfig::load();
    let binding_test = binding_test::RustBindingTest::new_in_tokio().unwrap();

    let rooch_key_pair = RoochKeyPair::generate_secp256k1();

    let mut test_transaction_builder = TestTransactionBuilder::new(rooch_key_pair);

    let tx_type = config.tx_type.unwrap().clone();

    let tx_cnt = 1000;
    let transactions: Vec<_> = (0..tx_cnt)
        .map(|_n| {
            // Because the validate function doesn't increase the sequence number,
            // use the sequence number 0 for all transactions
            create_l2_tx(&mut test_transaction_builder, 0, tx_type.clone()).unwrap()
        })
        .collect();
    let mut transactions_iter = transactions.into_iter().cycle();

    c.bench_function("tx_validate", |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.executor.validate_l2_tx(tx).unwrap()
        });
    });
}

criterion_group! {
    name = tx_validate_bench;
    config = configure_criterion(None).measurement_time(Duration::from_millis(5000));
    targets = tx_validate_benchmark
}

criterion_main!(tx_validate_bench);
