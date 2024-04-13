// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use rooch_benchmarks::helper::profiled;
use rooch_benchmarks::tx::{create_l2_tx, gen_sequencer};
use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use rooch_types::transaction::LedgerTxData;
use std::time::Duration;

pub fn tx_sequence_benchmark(c: &mut Criterion) {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let rooch_account = keystore.addresses()[0];
    let rooch_key_pair = keystore
        .get_key_pairs(&rooch_account, None)
        .unwrap()
        .pop()
        .expect("key pair should have value");
    let sequencer_keypair = rooch_key_pair.copy();
    let mut sequencer =
        gen_sequencer(sequencer_keypair, binding_test.executor().get_rooch_store()).unwrap();

    let mut test_transaction_builder = TestTransactionBuilder::new(rooch_account.into());
    let tx_cnt = 100;
    let transactions: Vec<_> = (0..tx_cnt)
        .map(|n| {
            let tx = create_l2_tx(&mut test_transaction_builder, &keystore, n).unwrap();
            LedgerTxData::L2Tx(tx.clone())
        })
        .collect();
    let mut transactions_iter = transactions.into_iter().cycle();

    c.bench_function("tx_sequence", |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            sequencer.sequence(tx.clone()).unwrap()
        });
    });
}

criterion_group! {
    name = tx_sequence_bench;
    config = profiled(None).measurement_time(Duration::from_millis(500));
    targets = tx_sequence_benchmark
}

criterion_main!(tx_sequence_bench);
