// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::binding_test;
use bitcoin::consensus::deserialize;
use bitcoin::{Block, OutPoint, Transaction, TxOut};
use hex::FromHex;
use moveos_types::access_path::AccessPath;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::object;
use moveos_types::state::MoveStructType;
use moveos_types::state_resolver::StateReader;
use moveos_types::transaction::MoveAction;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::bitcoin::ord::{Inscription, InscriptionID};
use rooch_types::bitcoin::types::{self, Header};
use rooch_types::bitcoin::utxo::{self, UTXO};
use rooch_types::into_address::IntoAddress;
use rooch_types::transaction::rooch::RoochTransactionData;
use tracing::debug;

#[test]
fn test_submit_block() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    // Mainnet block 00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7
    let block_hex = Vec::<u8>::from_hex("010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b0201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0804ffff001d026e04ffffffff0100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bdef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99c48b0e3f39c51347a19a5819651503a5ac00000000010000000321f75f3139a013f50f315b23b0c9a2b6eac31e2bec98e5891c924664889942260000000049483045022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067fa6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a5014081b5c01ffffffff79cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b84d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea5982f88cef651e9354f5791602369bf5a82a6cd61a62501fffffffffe09f5fe3ffbf5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e82000000004847304402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d33812a5ce01ffffffff01009d966b01000000434104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac00000000").unwrap();
    let height = 496u64;
    let block: Block = deserialize(&block_hex).unwrap();
    let bitcoin_txdata = block.txdata.clone();
    let block_hash = block.header.block_hash();
    let block_header: Header = block.header.into();

    let action = MoveAction::Function(
        rooch_types::bitcoin::light_client::BitcoinLightClientModule::create_submit_new_block_call(
            height,
            block.clone(),
        ),
    );
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    // let moveos = binding_test.reader_executor.moveos().read();
    let bitcoin_light_client_module = binding_test
        .as_module_binding::<rooch_types::bitcoin::light_client::BitcoinLightClientModule>(
    );
    assert_eq!(
        bitcoin_light_client_module
            .get_block(block_hash)
            .unwrap()
            .unwrap(),
        block_header
    );

    assert_eq!(
        bitcoin_light_client_module
            .get_block_by_height(height)
            .unwrap()
            .unwrap(),
        block_header
    );
    assert_eq!(
        bitcoin_light_client_module
            .get_latest_block_height()
            .unwrap()
            .unwrap(),
        height
    );
    println!("txdata len: {}", bitcoin_txdata.len());

    assert!(bitcoin_light_client_module.remaining_tx_count().unwrap() > 0);
    let sequence_number = sequence_number + 1;
    let tx_data = RoochTransactionData::new_for_test(
        sender,
        sequence_number,
        MoveAction::Function(
            rooch_types::bitcoin::light_client::BitcoinLightClientModule::create_process_utxos_call(
                bitcoin_txdata.len() as u64,
            ),
        ),
    );
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    check_utxo(bitcoin_txdata, &binding_test);

    let timestamp_module =
        binding_test.as_module_binding::<rooch_types::framework::timestamp::TimestampModule>();

    let now_milliseconds = timestamp_module.now_milliseconds().unwrap();
    let duration = std::time::Duration::from_secs(block_header.time as u64);
    println!(
        "now_milliseconds: {}, header_timestamp: {}",
        now_milliseconds, block_header.time as u64
    );
    assert_eq!(now_milliseconds, duration.as_millis() as u64);
}

//we temporarily ignore this test because it takes too long time
//to run this test, use command:
//RUST_LOG=debug cargo test --release --package rooch-framework-tests --lib -- --include-ignored tests::bitcoin_light_client_test::test_utxo_progress
#[ignore]
#[test]
fn test_utxo_progress() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let mut sequence_number = 0;

    let btc_block_hex = include_str!("../blocks/818677.txt");
    let btc_block_bytes = Vec::<u8>::from_hex(btc_block_hex).unwrap();
    let height = 818677u64;
    let block: Block = deserialize(&btc_block_bytes).unwrap();

    let action = MoveAction::Function(
        rooch_types::bitcoin::light_client::BitcoinLightClientModule::create_submit_new_block_call(
            height,
            block.clone(),
        ),
    );
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let bitcoin_light_client_module = binding_test
        .as_module_binding::<rooch_types::bitcoin::light_client::BitcoinLightClientModule>(
    );
    assert!(bitcoin_light_client_module.remaining_tx_count().unwrap() > 0);
    let mut remaining_tx_count = bitcoin_light_client_module.remaining_tx_count().unwrap();
    while remaining_tx_count > 0 {
        sequence_number = sequence_number + 1;
        let tx_data = RoochTransactionData::new_for_test(
        sender,
        sequence_number,
        MoveAction::Function(
            rooch_types::bitcoin::light_client::BitcoinLightClientModule::create_process_utxos_call(
                1000,
            ),
        ),
        );
        let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
        binding_test.execute(tx).unwrap();
        let bitcoin_light_client_module =
            binding_test
                .as_module_binding::<rooch_types::bitcoin::light_client::BitcoinLightClientModule>(
                );
        remaining_tx_count = bitcoin_light_client_module.remaining_tx_count().unwrap();
    }
    check_utxo(block.txdata, &binding_test);
}

fn check_utxo(txs: Vec<Transaction>, binding_test: &binding_test::RustBindingTest) {
    let mut utxo_set = HashMap::<OutPoint, TxOut>::new();
    for tx in txs.as_slice() {
        for (index, tx_out) in tx.output.iter().enumerate() {
            let vout = index as u32;
            let out_point = OutPoint::new(tx.txid(), vout);
            utxo_set.insert(out_point, tx_out.clone());
        }
        for tx_in in tx.input.iter() {
            utxo_set.remove(&tx_in.previous_output);
        }
    }

    let utxo_module = binding_test.as_module_binding::<rooch_types::bitcoin::utxo::UTXOModule>();

    let moveos_resolver = binding_test.executor().moveos().moveos_resolver();

    for (outpoint, tx_out) in utxo_set.into_iter() {
        let outpoint: types::OutPoint = outpoint.into();
        debug!("check utxo: outpoint {}", outpoint);
        assert!(
            utxo_module.exists_utxo(&outpoint).unwrap(),
            "Can not find utxo: outpoint {} from utxo_module",
            outpoint
        );

        let utxo_id = utxo::derive_utxo_id(&outpoint);
        let utxo_state = moveos_resolver
            .get_states(AccessPath::object(utxo_id))
            .unwrap()
            .pop()
            .unwrap();
        assert!(
            utxo_state.is_some(),
            "Can not find utxo object for outpoint: {}",
            outpoint,
        );
        let utxo_state = utxo_state.unwrap();
        let utxo_object = utxo_state.as_object::<UTXO>().unwrap();
        assert_eq!(utxo_object.value.txid, outpoint.txid);
        assert_eq!(utxo_object.value.vout, outpoint.vout);
        assert_eq!(utxo_object.value.value, tx_out.value.to_sat());
    }

    let inscriptions = txs
        .iter()
        .map(|tx| {
            let txid = tx.txid();
            bitcoin_move::natives::ord::from_transaction(tx)
                .into_iter()
                .enumerate()
                .map(move |(idx, i)| ((txid.clone(), idx, i)))
        })
        .flatten()
        .collect::<Vec<_>>();
    for (txid, index, inscription) in inscriptions {
        let txid_address = txid.into_address();
        let index = index as u32;
        debug!(
            "check inscription: txid: {}, index: {}",
            txid_address, index
        );
        let inscription_id = InscriptionID::new(txid_address, index);
        let object_id = object::custom_object_id(&inscription_id, &Inscription::struct_tag());
        let inscription_state = moveos_resolver
            .get_states(AccessPath::object(object_id))
            .unwrap()
            .pop()
            .unwrap();
        assert!(
            inscription_state.is_some(),
            "Can not find inscription: txid: {}, index: {}",
            txid_address,
            index
        );
        let inscription_state = inscription_state.unwrap();
        let inscription_object = inscription_state.as_object::<Inscription>().unwrap();
        assert_eq!(inscription_object.value.txid, txid.into_address());
        assert_eq!(inscription_object.value.index, index);
        assert_eq!(
            inscription_object.value.body,
            inscription.body.unwrap_or_default()
        );
    }
}
