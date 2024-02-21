// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use moveos_config::temp_dir;
use rooch_benchmarks::tx::{create_publish_transaction, create_transaction, setup_service};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn transaction_write_benchmark(c: &mut Criterion) {
    let tempdir = temp_dir();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let rt: Runtime = Runtime::new().unwrap();
    let (rpc_service, _aggregate_service) =
        rt.block_on(async { setup_service(&tempdir, &keystore).await.unwrap() });

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());
    let tx = create_publish_transaction(&test_transaction_builder, &keystore).unwrap();
    let _publish_result = rt.block_on(async { rpc_service.execute_tx(tx).await.unwrap() });

    let mut transactions = (1..500)
        .cycle()
        .map(|n| create_transaction(&mut test_transaction_builder, &keystore, n).unwrap());
    c.bench_function("execute_tx", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| rpc_service.execute_tx(transactions.next().unwrap()))
    });
}

criterion_group! {
    name = rooch_tx_write_bench;
    config = Criterion::default().sample_size(200).measurement_time(Duration::from_secs(100));
    targets = transaction_write_benchmark
}

criterion_main!(rooch_tx_write_bench);
