// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use moveos_config::temp_dir;
use rooch_benchmarks::tx::{create_publish_transaction, create_transaction, setup_service};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_rpc_api::api::rooch_api::RoochAPIServer;
use rooch_rpc_api::jsonrpc_types::StrView;
use rooch_rpc_server::server::rooch_server::RoochServer;
use rooch_test_transaction_builder::TestTransactionBuilder;
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn transaction_query_benchmark(c: &mut Criterion) {
    let tempdir = temp_dir();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let rt: Runtime = Runtime::new().unwrap();
    let (rpc_service, aggregate_service) =
        rt.block_on(async { setup_service(&tempdir, &keystore).await.unwrap() });
    let rooch_server = RoochServer::new(rpc_service.clone(), aggregate_service);

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());
    let tx = create_publish_transaction(&test_transaction_builder, &keystore).unwrap();
    let _publish_result = rt.block_on(async { rpc_service.execute_tx(tx).await.unwrap() });
    //
    for n in 1..500 {
        let tx = create_transaction(&mut test_transaction_builder, &keystore, n).unwrap();
        let _ = rt.block_on(async { rpc_service.execute_tx(tx).await.unwrap() });
    }

    let mut tx_orders = (1..500).cycle().map(|v| v);
    c.bench_function("get_transactions_by_order", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| {
            rooch_server.get_transactions_by_order(
                Some(StrView(tx_orders.next().unwrap())),
                None,
                None,
            )
        })
    });
}

criterion_group! {
    name = rooch_tx_query_bench;
    config = Criterion::default().sample_size(200).measurement_time(Duration::from_secs(10));
    targets = transaction_query_benchmark
}
criterion_main!(rooch_tx_query_bench);
