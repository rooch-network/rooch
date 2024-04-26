// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{Criterion, SamplingMode};

use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;

use crate::config::BenchTxConfig;
use crate::config::TxType::{BtcBlock, Empty, Transfer};
use crate::tx::{create_btc_blk_tx, create_l2_tx, find_block_height};

// pure execution, no validate, sequence
pub fn tx_exec_benchmark(c: &mut Criterion) {
    let config = BenchTxConfig::load();

    let mut binding_test =
        binding_test::RustBindingTest::new_with_mode(config.data_import_mode.unwrap().to_num())
            .unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);
    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());

    let tx_type = config.tx_type.unwrap();
    let (bench_id, tx_cnt) = match tx_type {
        BtcBlock => ("btc_blk", 20), // block after 800,000 always need seconds/block
        Transfer => ("l2_tx_transfer", 800),
        Empty => ("l2_tx_empty", 1000),
    };

    let mut transactions: Vec<_> = Vec::with_capacity(tx_cnt);
    if tx_type != BtcBlock {
        for n in 0..tx_cnt {
            let tx = create_l2_tx(
                &mut test_transaction_builder,
                &keystore,
                n as u64,
                tx_type.clone(),
            )
            .unwrap();
            transactions.push(binding_test.executor.validate_l2_tx(tx.clone()).unwrap());
        }
    } else {
        let btc_blk_dir = config.btc_block_dir.clone().unwrap();

        let heights = find_block_height(btc_blk_dir.clone()).unwrap();
        for (cnt, height) in heights.into_iter().enumerate() {
            if cnt >= tx_cnt {
                break;
            }
            let filename = format!("{}.hex", height);
            let file_path = [btc_blk_dir.clone(), "/".parse().unwrap(), filename].concat();
            let l1_block = create_btc_blk_tx(height, file_path).unwrap();
            let ctx = binding_test.create_bt_blk_tx_ctx(cnt as u64, l1_block.clone());
            let move_tx = binding_test
                .executor
                .validate_l1_block(ctx, l1_block.clone())
                .unwrap();
            transactions.push(move_tx);
        }
    }

    let mut transactions_iter = transactions.into_iter().cycle();

    let mut group = c.benchmark_group("bench_tx_exec");
    group.sample_size(tx_cnt);
    group.sampling_mode(SamplingMode::Flat);
    group.bench_function(bench_id, |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.execute_verified_tx(tx.clone()).unwrap()
        });
    });
    group.finish();
}
