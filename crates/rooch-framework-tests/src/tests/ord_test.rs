// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use bitcoin::consensus::{deserialize, Decodable};
use bitcoin::Block;
use hex::FromHex;
use moveos_types::state::MoveState;
use moveos_types::transaction::MoveAction;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::framework::ord::Inscription;
use rooch_types::transaction::rooch::RoochTransactionData;

#[test]
fn test_from_transaction() {
    tracing_subscriber::fmt::init();
    let binding_test = binding_test::RustBindingTest::new().unwrap();

    //https://mempool.space/api/tx/a03c44005f1871a6068e286a4151b009e3f6184983464782820b56633760333d/hex
    let btc_tx_hex = "02000000000101361cc743a923abc1db73f4fed4d0778cc8ccc092cb20f1c66cada177818e55b20000000000fdffffff022202000000000000225120e5053d2151d14399a3a4825740e14deae6f984e990e0a6872df065a6dad7009c6e04000000000000160014ad45c620bd9b6688c5a7a23e515402d39d02b55203401500c4f407f66ec47c92e1daf34c46f2b52837819119b696e343385b6dba27682dd89f9e4d18354ce0f4a4200ddab8420457392702e1e0b6d51803d25d2bf2647f2016c3a3f18eb4efd24274941ba02c899d151b0473a1bad3512423cbe1b0648ea9ac0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800397b2270223a226272632d3230222c226f70223a227472616e73666572222c227469636b223a226f726469222c22616d74223a2231303030227d6821c102a58d972468a33a79350cf24cb991f28adbbe3e64e88ded5f58f558fff2b67300000000";
    let btc_tx_bytes = Vec::from_hex(btc_tx_hex).unwrap();
    let btc_tx: bitcoin::Transaction =
        Decodable::consensus_decode(&mut btc_tx_bytes.as_slice()).unwrap();
    let inscriptions =
        rooch_framework::natives::rooch_framework::bitcoin::ord::from_transaction(&btc_tx);
    //print!("{:?}", inscriptions);
    let ord_module = binding_test.as_module_bundle::<rooch_types::framework::ord::OrdModule>();
    let move_btc_tx: rooch_types::framework::bitcoin_types::Transaction = btc_tx.into();
    //println!("tx_hex: {}", hex::encode(move_btc_tx.to_bytes()));
    let inscriptions_from_move = ord_module.from_transaction(&move_btc_tx).unwrap();
    assert_eq!(inscriptions.len(), inscriptions_from_move.len());
    for (inscription, inscription_from_move) in inscriptions.into_iter().zip(inscriptions_from_move)
    {
        assert_eq!(Inscription::from(inscription), inscription_from_move);
    }
}

#[test]
fn test_ord_module() {
    tracing_subscriber::fmt::try_init().unwrap();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    let btc_block_hex = include_str!("../blocks/818677.txt");
    let btc_block_bytes = Vec::<u8>::from_hex(btc_block_hex).unwrap();
    let height = 818677u64;
    let block: Block = deserialize(&btc_block_bytes).unwrap();
    let bitcoin_txdata = block.txdata.clone();

    let inscriptions = bitcoin_txdata
        .iter()
        .map(|tx| rooch_framework::natives::rooch_framework::bitcoin::ord::from_transaction(tx))
        .flatten()
        .collect::<Vec<_>>();
    //println!("inscriptions: {:?}", inscriptions.len());
    let action = MoveAction::Function(rooch_types::framework::bitcoin_light_client::BitcoinLightClientModule::create_submit_new_block_call(height, block.clone()));
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let ord_module = binding_test.as_module_bundle::<rooch_types::framework::ord::OrdModule>();
    assert!(ord_module.remaining_tx_count().unwrap() > 0);

    let sequence_number = sequence_number + 1;
    let tx_data = RoochTransactionData::new_for_test(
        sender,
        sequence_number,
        MoveAction::Function(
            rooch_types::framework::ord::OrdModule::create_progress_inscriptions_call(
                bitcoin_txdata.len() as u64,
            ),
        ),
    );
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let ord_module = binding_test.as_module_bundle::<rooch_types::framework::ord::OrdModule>();
    let total_inscriptions_in_move = ord_module.total_inscriptions().unwrap();
    assert_eq!(total_inscriptions_in_move, inscriptions.len() as u64);
}
