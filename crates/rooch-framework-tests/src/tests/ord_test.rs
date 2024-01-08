// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use bitcoin::consensus::Decodable;
use hex::FromHex;
use moveos_types::module_binding::MoveFunctionCaller;
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
    let inscriptions = bitcoin_move::natives::ord::from_transaction(&btc_tx);

    let ord_module = binding_test.as_module_binding::<rooch_types::bitcoin::ord::OrdModule>();
    let move_btc_tx: rooch_types::bitcoin::types::Transaction =
        rooch_types::bitcoin::types::Transaction::from(btc_tx);
    let inscriptions_from_move = ord_module.from_transaction(&move_btc_tx).unwrap();

    for (i, (inscription, inscription_from_move)) in inscriptions
        .into_iter()
        .zip(inscriptions_from_move)
        .enumerate()
    {
        debug!("{}. inscription: {:?}", i, inscription);
        assert_eq!(
            inscription.body.unwrap_or_default(),
            inscription_from_move.body
        );
    }
}

#[test]
fn test_from_transaction() {
    let _ = tracing_subscriber::fmt::try_init();

    //https://mempool.space/api/tx/69d52ccb5eb80372b7fc6c4fc3feb17038dd2f58313c5d16302d70f7ef0fff7f/hex
    let btc_tx_hex = "02000000000101c5d8fc62b7512401b6fc31911dbb763a473d6e1cfd63966f348d617c2f7b721c0100000000fffffffd02260100000000000016001480f177474e4e9caba5eb4d58f6d071264401d072ab6919020000000022512074a7bcc1ed5fc5b28680a838ccf8e745fd8afdcb171a95e4e1bcf0100792c3e103408160d443c29618f76a7498f952c8b42309489566f482cd9564796720056d948b86b84bcf44c1068ca6146a611a592f3f6799b55dd4682859b992d6a11d48d28d452088225c0158a85208c9f0a93d3b724953f164a056121b81a87f88ab0a666cbff1ac0063036f726401010a746578742f706c61696e000d3832303936302e6269746d61706821c188225c0158a85208c9f0a93d3b724953f164a056121b81a87f88ab0a666cbff100000000";
    let _inscriptions_from_move = decode_inscription(btc_tx_hex);
}

#[test]
fn test_local_tx() {
    let _ = tracing_subscriber::fmt::try_init();
    //commit tx
    let btx_tx_hex = "01000000000101303a8b5191266d37103f6c4c6033019b59d98ac468e07f61ddbc6f50b204a7eb0000000000fdffffff029b270000000000002251201d44f728e28f6ffa0b0094edabefb466a348e1e7adbf2ff0e7e70abd2ed8871bcbd0029500000000225120ae68b97d450930db183dede9c33fbb1c147a69672ecc3100a75f9be25c50f0cf014064cf4601628581d59a8dc482f10c67106ffad818869e733d9cabbdab73240da556aa4d04afd5fa1e72d5ca1ea4fa151385e3e8a7596ebd3959212c2a57696c8400000000";
    decode_inscription(btx_tx_hex);
    //reveal tx
    let btc_tx_hex = "010000000001019cea25cbdacc895f9dbb85e4bfb7aa51d04cc69cc7f75ed49da3ff3f442f2e7f0000000000fdffffff01102700000000000022512036646c76dd6505025341c7cc1cf6c22fcc638c47454945da1948a4637a86f9200340e7f99517f921be44b83854b05d7eb98f7c1a9a0cd373ade7e29c1bcc321190bef1766976726641b69ccc61f9ea62ea55b97e644ec47a15481d3b70b0c44d008f4c207f6ef96528b25ace707fe33f4a23113c824da971dab921a2ad311c309edf0944ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d38000668656c6c6f0a6821c17f6ef96528b25ace707fe33f4a23113c824da971dab921a2ad311c309edf094400000000";
    decode_inscription(btc_tx_hex);
}
