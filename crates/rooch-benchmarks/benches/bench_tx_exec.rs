// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use rooch_benchmarks::helper::profiled;
use rooch_benchmarks::tx::TxType::{Blog, Empty, Transfer};
use rooch_benchmarks::tx::{create_publish_transaction, create_transaction, TX_TYPE};
use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;

pub fn tx_exec_benchmark(c: &mut Criterion) {
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());
    let mut tx_cnt = 300;
    if *TX_TYPE == Blog {
        let tx = create_publish_transaction(&test_transaction_builder, &keystore).unwrap();
        binding_test.execute(tx).unwrap();
    }
    if *TX_TYPE == Transfer {
        tx_cnt = 500;
    }
    if *TX_TYPE == Empty {
        tx_cnt = 1000;
    }
    let transactions: Vec<_> = (0..tx_cnt)
        .map(|n| {
            let tx = create_transaction(&mut test_transaction_builder, &keystore, n).unwrap();
            binding_test.executor.validate_l2_tx(tx.clone()).unwrap()
        })
        .collect();
    let mut transactions_iter = transactions.into_iter().cycle();

    c.bench_function("tx_exec", |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.execute_verified_tx(tx.clone()).unwrap()
        });
    });
}

criterion_group! {
    name = tx_exec_bench;
    config = profiled(None);
    targets = tx_exec_benchmark
}

criterion_main!(tx_exec_bench);
