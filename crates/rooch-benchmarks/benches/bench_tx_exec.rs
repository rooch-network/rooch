// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use rooch_benchmarks::helper::profiled;
use rooch_benchmarks::tx::TxType::{BTCBlk, Blog, Empty, Transfer};
use rooch_benchmarks::tx::{
    create_btc_blk_tx, create_l2_tx, create_publish_transaction, find_block_height,
    tx_exec_benchmark, BTC_BLK_DIR, TX_TYPE,
};
use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use rooch_types::transaction::L1BlockWithBody;

criterion_group! {
    name = tx_exec_bench;
    config = profiled(None);
    targets = tx_exec_benchmark
}

criterion_main!(tx_exec_bench);
