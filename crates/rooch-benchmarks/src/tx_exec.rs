use crate::tx::TxType::{BTCBlk, Blog, Empty, Transfer};
use crate::tx::{
    create_btc_blk_tx, create_l2_tx, create_publish_transaction, find_block_height, IMPORT_MODE,
    TX_TYPE,
};
use criterion::{Criterion, SamplingMode};
use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use std::env;

// pure execution, no validate, sequence
pub fn tx_exec_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat-sampling-example");
    group.sampling_mode(SamplingMode::Flat);
    let mut binding_test =
        binding_test::RustBindingTest::new_with_mode((*IMPORT_MODE).to_num()).unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());

    let mut tx_cnt = 300;
    let mut bench_id = "tx_exec";

    match *TX_TYPE {
        Blog => {
            let tx = create_publish_transaction(&test_transaction_builder, &keystore).unwrap();
            binding_test.execute(tx).unwrap();
        }
        BTCBlk => {
            bench_id = "tx_exec_blk";
            tx_cnt = 200
        }
        Transfer => tx_cnt = 500,
        Empty => tx_cnt = 1000,
    }

    let mut transactions: Vec<_> = Vec::with_capacity(tx_cnt);
    if *TX_TYPE != BTCBlk {
        for n in 0..tx_cnt {
            let tx = create_l2_tx(&mut test_transaction_builder, &keystore, n as u64).unwrap();
            transactions.push(binding_test.executor.validate_l2_tx(tx.clone()).unwrap());
        }
    } else {
        let btc_blk_dir = env::var("ROOCH_BENCH_BTC_BLK_DIR").unwrap();

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

    group.bench_function(bench_id, |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.execute_verified_tx(tx.clone()).unwrap()
        });
    });
    group.finish();
}
