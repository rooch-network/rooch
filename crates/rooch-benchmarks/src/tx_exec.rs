// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::BenchTxConfig;
use crate::config::TxType::{BtcBlock, BtcTx, Empty, Transfer};
use crate::tx::{create_btc_blk_tx, create_l2_tx, find_block_height, prepare_btc_block};
use criterion::{Criterion, SamplingMode};
use rooch_framework_tests::binding_test;
use rooch_test_transaction_builder::TestTransactionBuilder;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::transaction::LedgerTxData;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

// pure execution, no validate, sequence
pub fn tx_exec_benchmark(c: &mut Criterion) {
    let config = BenchTxConfig::load();

    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    let kp = RoochKeyPair::generate_secp256k1();
    let mut test_transaction_builder = TestTransactionBuilder::new(kp);

    let tx_type = config.tx_type.clone().unwrap();
    let (bench_id, tx_cnt) = match tx_type {
        BtcBlock => ("btc_block", 100),
        BtcTx => ("btc_tx", 5), // The tx_cnt is the block count
        Transfer => ("l2_tx_transfer", 800),
        Empty => ("l2_tx_empty", 1000),
    };
    let mut blocks = HashMap::new();
    let mut transactions: Vec<_> = Vec::with_capacity(tx_cnt);
    match tx_type {
        BtcBlock | BtcTx => {
            let btc_blk_dir = PathBuf::from(config.btc_block_dir.clone().unwrap());
            //if the btc block dir is not absolute, we will treat it as relative to the workspace dir
            let btc_blk_dir = if !btc_blk_dir.is_absolute() {
                let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_path_buf();
                workspace_dir.join(btc_blk_dir)
            } else {
                btc_blk_dir
            };

            if config.btc_rpc_url.is_some()
                && config.btc_rpc_username.is_some()
                && config.btc_rpc_password.is_some()
            {
                prepare_btc_block(
                    &btc_blk_dir,
                    config.btc_rpc_url.clone().unwrap(),
                    config.btc_rpc_username.clone().unwrap(),
                    config.btc_rpc_password.clone().unwrap(),
                    config.btc_block_start_height.unwrap(),
                    tx_cnt as u64,
                );
            } else {
                info!("The btc rpc url or user or pass is not set, skip prepare btc block");
            }

            let heights = find_block_height(&btc_blk_dir).unwrap();
            if heights.is_empty() {
                panic!("No btc block data found in {:?}, please set the btc rpc config to prepare the data.", btc_blk_dir);
            }
            for (cnt, height) in heights.into_iter().enumerate() {
                if cnt >= tx_cnt {
                    break;
                }
                let filename = format!("{}.hex", height);
                let file_path = btc_blk_dir.join(filename);
                let l1_block = create_btc_blk_tx(height, &file_path).unwrap();
                if tx_type == BtcBlock {
                    transactions.push(LedgerTxData::L1Block(l1_block.block.clone()));
                    blocks.insert(l1_block.block.block_height, l1_block);
                } else {
                    let l1_txs = binding_test.execute_l1_block(l1_block).unwrap();
                    for tx in l1_txs {
                        transactions.push(LedgerTxData::L1Tx(tx));
                    }
                }
            }
        }
        _ => {
            for n in 0..tx_cnt {
                let tx =
                    create_l2_tx(&mut test_transaction_builder, n as u64, tx_type.clone()).unwrap();
                transactions.push(LedgerTxData::L2Tx(tx));
            }
        }
    }
    let sample_size = transactions.len();
    let mut transactions_iter = transactions.into_iter();

    let mut group = c.benchmark_group("bench_tx_exec");
    group.sample_size(sample_size);
    group.sampling_mode(SamplingMode::Flat);
    group.bench_function(bench_id, |b| {
        b.iter(|| {
            let tx = match transactions_iter.next() {
                Some(tx) => tx,
                None => {
                    //TODO we can not use `transactions.into_iter().cycle()` to repeat the txs, because the sequence number can not be repeated
                    return;
                }
            };
            match tx {
                LedgerTxData::L1Block(l1_block) => {
                    let l1_block_with_body = blocks.get(&l1_block.block_height).unwrap();
                    binding_test
                        .execute_l1_block(l1_block_with_body.clone())
                        .unwrap();
                }
                LedgerTxData::L1Tx(tx) => binding_test.execute_l1_tx(tx).unwrap(),
                LedgerTxData::L2Tx(tx) => binding_test.execute(tx).unwrap(),
            }
        });
    });
    group.finish();
}
