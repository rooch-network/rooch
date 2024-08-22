// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    binding_test,
    tests::bitcoin_data::{bitcoin_tx_from_hex, load_tx, load_tx_info, TxInfo},
};
use bitcoin::Transaction;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_types::bitcoin::network::Network;
use rooch_types::bitcoin::ord::Inscription;
use tracing::debug;

fn decode_inscription(
    binding_test: &mut binding_test::RustBindingTest,
    btc_tx: Transaction,
    input_utxo_values: Vec<u64>,
    next_inscription_number: u32,
    next_sequence_number: u32,
) -> Vec<Inscription> {
    debug!("tx_id: {}", btc_tx.txid());
    for (i, input) in btc_tx.input.iter().enumerate() {
        debug!("{}. input: {:?}", i, input.previous_output);
    }
    for (i, output) in btc_tx.output.iter().enumerate() {
        debug!(
            "{}. output: {:?}, public_key: {:?}",
            i,
            output,
            output.script_pubkey.p2wpkh_script_code()
        );
    }

    let ord_module = binding_test.as_module_binding::<rooch_types::bitcoin::ord::OrdModule>();
    let move_btc_tx: rooch_types::bitcoin::types::Transaction =
        rooch_types::bitcoin::types::Transaction::from(btc_tx);

    ord_module
        .from_transaction(
            &move_btc_tx,
            input_utxo_values,
            next_inscription_number,
            next_sequence_number,
        )
        .unwrap()
}

#[tokio::test]
async fn test_8706753() {
    //https://ordiscan.com/inscription/8706753
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    //https://mempool.space/api/tx/4b8111663106c242da8580ba38c36f261287b9c35b1aa5974f4c14306905e720/hex
    let btc_tx = load_tx(
        Network::Bitcoin,
        "4b8111663106c242da8580ba38c36f261287b9c35b1aa5974f4c14306905e720",
    );
    let input_utxo_values = vec![3318u64];
    let next_inscription_number = 8706753;
    let next_sequence_number = 8709019;
    let inscribe_tx_id = btc_tx.txid();
    let mut inscriptions = decode_inscription(
        &mut binding_test,
        btc_tx,
        input_utxo_values,
        next_inscription_number,
        next_sequence_number,
    );

    assert_eq!(inscriptions.len(), 1);
    let inscription = inscriptions.pop().unwrap();
    let object_id = inscription.object_id();

    let ord_module = binding_test.as_module_binding::<rooch_types::bitcoin::ord::OrdModule>();
    //https://mempool.space/api/tx/e5efc3b2bbf3d738d253e62ffde36b51abb5d12a748abf89d06fca456345fe48
    let btc_tx_info: TxInfo = load_tx_info(
        Network::Bitcoin,
        "e5efc3b2bbf3d738d253e62ffde36b51abb5d12a748abf89d06fca456345fe48",
    );
    let input_index = btc_tx_info
        .vin
        .iter()
        .position(|input| input.txid == inscribe_tx_id)
        .unwrap() as u64;
    let input_utxo_values: Vec<u64> = btc_tx_info
        .vin
        .iter()
        .map(|input| input.prevout.value.to_sat())
        .collect();
    let spend_tx: Transaction = btc_tx_info.into();
    //let expect_offset = 316084756u64;
    let (is_match, sat_point) = ord_module
        .match_utxo_and_generate_sat_point(
            inscription.offset,
            object_id,
            &spend_tx.into(),
            input_utxo_values,
            input_index,
        )
        .unwrap();
    debug!("is_match: {}, sat_point: {:?}", is_match, sat_point);
    //The inscription is spent via fee, so the is_match should be false
    assert!(!is_match);
    //TODO how to verify the coinbase sat_point
}

//RUST_LOG=debug cargo test test_from_tx -- --nocapture
#[tokio::test]
async fn test_from_tx() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    //https://mempool.space/api/tx/69d52ccb5eb80372b7fc6c4fc3feb17038dd2f58313c5d16302d70f7ef0fff7f/hex
    let btc_tx = load_tx(
        Network::Bitcoin,
        "69d52ccb5eb80372b7fc6c4fc3feb17038dd2f58313c5d16302d70f7ef0fff7f",
    );
    decode_inscription(&mut binding_test, btc_tx, vec![], 0, 0);
}

#[tokio::test]
async fn test_local_tx() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    //commit tx
    let btx_tx = bitcoin_tx_from_hex("01000000000101303a8b5191266d37103f6c4c6033019b59d98ac468e07f61ddbc6f50b204a7eb0000000000fdffffff029b270000000000002251201d44f728e28f6ffa0b0094edabefb466a348e1e7adbf2ff0e7e70abd2ed8871bcbd0029500000000225120ae68b97d450930db183dede9c33fbb1c147a69672ecc3100a75f9be25c50f0cf014064cf4601628581d59a8dc482f10c67106ffad818869e733d9cabbdab73240da556aa4d04afd5fa1e72d5ca1ea4fa151385e3e8a7596ebd3959212c2a57696c8400000000");
    decode_inscription(&mut binding_test, btx_tx, vec![], 0, 0);
    //reveal tx
    let btc_tx = bitcoin_tx_from_hex("010000000001019cea25cbdacc895f9dbb85e4bfb7aa51d04cc69cc7f75ed49da3ff3f442f2e7f0000000000fdffffff01102700000000000022512036646c76dd6505025341c7cc1cf6c22fcc638c47454945da1948a4637a86f9200340e7f99517f921be44b83854b05d7eb98f7c1a9a0cd373ade7e29c1bcc321190bef1766976726641b69ccc61f9ea62ea55b97e644ec47a15481d3b70b0c44d008f4c207f6ef96528b25ace707fe33f4a23113c824da971dab921a2ad311c309edf0944ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d38000668656c6c6f0a6821c17f6ef96528b25ace707fe33f4a23113c824da971dab921a2ad311c309edf094400000000");
    decode_inscription(&mut binding_test, btc_tx, vec![], 0, 0);
}
