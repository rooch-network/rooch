// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use rooch_benchmarks::config::{configure_criterion, BenchTxConfig};
use rooch_benchmarks::tx::create_l2_tx;
use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;

pub fn tx_validate_benchmark(c: &mut Criterion) {
    let mut config = BenchTxConfig::load();
    config.adjust();

    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());

    let tx_type = config.tx_type.unwrap().clone();

    let tx_cnt = 600;
    let transactions: Vec<_> = (0..tx_cnt)
        .map(|n| {
            create_l2_tx(&mut test_transaction_builder, &keystore, n, tx_type.clone()).unwrap()
        })
        .collect();
    let mut transactions_iter = transactions.into_iter().cycle();

    c.bench_function("tx_validate", |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.executor.validate_l2_tx(tx.clone()).unwrap()
        });
    });
}

criterion_group! {
    name = tx_validate_bench;
    config = configure_criterion(None).measurement_time(Duration::from_millis(200));
    targets = tx_validate_benchmark
}

criterion_main!(tx_validate_bench);
