// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use bitcoin::consensus::Decodable;
use hex::FromHex;
use rooch_types::framework::ord::InscriptionRecord;
use tracing::debug;

fn decode_inscription(btx_tx_hex: &str) {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let btc_tx_bytes = Vec::from_hex(btx_tx_hex).unwrap();
    let btc_tx: bitcoin::Transaction =
        Decodable::consensus_decode(&mut btc_tx_bytes.as_slice()).unwrap();
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
    let inscriptions =
        rooch_framework::natives::rooch_framework::bitcoin::ord::from_transaction(&btc_tx);

    let ord_module = binding_test.as_module_bundle::<rooch_types::framework::ord::OrdModule>();
    let move_btc_tx: rooch_types::framework::bitcoin_types::Transaction = btc_tx.into();
    let inscriptions_from_move = ord_module.from_transaction(&move_btc_tx).unwrap();

    for (i, (inscription, inscription_from_move)) in inscriptions
        .into_iter()
        .zip(inscriptions_from_move)
        .enumerate()
    {
        debug!("{}. inscription: {:?}", i, inscription);
        assert_eq!(InscriptionRecord::from(inscription), inscription_from_move);
    }
}

#[test]
fn test_from_transaction() {
    tracing_subscriber::fmt::init();

    //https://mempool.space/api/tx/a03c44005f1871a6068e286a4151b009e3f6184983464782820b56633760333d/hex
    let btc_tx_hex = "02000000000101361cc743a923abc1db73f4fed4d0778cc8ccc092cb20f1c66cada177818e55b20000000000fdffffff022202000000000000225120e5053d2151d14399a3a4825740e14deae6f984e990e0a6872df065a6dad7009c6e04000000000000160014ad45c620bd9b6688c5a7a23e515402d39d02b55203401500c4f407f66ec47c92e1daf34c46f2b52837819119b696e343385b6dba27682dd89f9e4d18354ce0f4a4200ddab8420457392702e1e0b6d51803d25d2bf2647f2016c3a3f18eb4efd24274941ba02c899d151b0473a1bad3512423cbe1b0648ea9ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800397b2270223a226272632d3230222c226f70223a227472616e73666572222c227469636b223a226f726469222c22616d74223a2231303030227d6821c102a58d972468a33a79350cf24cb991f28adbbe3e64e88ded5f58f558fff2b67300000000";
    let _inscriptions_from_move = decode_inscription(btc_tx_hex);
}

#[test]
fn test_local_tx() {
    //commit tx
    let btx_tx_hex = "01000000000101303a8b5191266d37103f6c4c6033019b59d98ac468e07f61ddbc6f50b204a7eb0000000000fdffffff029b270000000000002251201d44f728e28f6ffa0b0094edabefb466a348e1e7adbf2ff0e7e70abd2ed8871bcbd0029500000000225120ae68b97d450930db183dede9c33fbb1c147a69672ecc3100a75f9be25c50f0cf014064cf4601628581d59a8dc482f10c67106ffad818869e733d9cabbdab73240da556aa4d04afd5fa1e72d5ca1ea4fa151385e3e8a7596ebd3959212c2a57696c8400000000";
    decode_inscription(btx_tx_hex);
    //reveal tx
    let btc_tx_hex = "010000000001019cea25cbdacc895f9dbb85e4bfb7aa51d04cc69cc7f75ed49da3ff3f442f2e7f0000000000fdffffff01102700000000000022512036646c76dd6505025341c7cc1cf6c22fcc638c47454945da1948a4637a86f9200340e7f99517f921be44b83854b05d7eb98f7c1a9a0cd373ade7e29c1bcc321190bef1766976726641b69ccc61f9ea62ea55b97e644ec47a15481d3b70b0c44d008f4c207f6ef96528b25ace707fe33f4a23113c824da971dab921a2ad311c309edf0944ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d38000668656c6c6f0a6821c17f6ef96528b25ace707fe33f4a23113c824da971dab921a2ad311c309edf094400000000";
    decode_inscription(btc_tx_hex);
}
