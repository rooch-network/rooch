// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use crate::tests::bitcoin_data::load_block;
use bitcoin::consensus::deserialize;
use bitcoin::hashes::Hash;
use bitcoin::{Block, OutPoint, Transaction, TxOut};
use hex::FromHex;
use moveos_types::access_path::AccessPath;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::state_resolver::StateReader;
use rooch_types::bitcoin::network::Network;
use rooch_types::bitcoin::ord;
use rooch_types::bitcoin::ord::{Inscription, InscriptionID};
use rooch_types::bitcoin::types::{self, Header};
use rooch_types::bitcoin::utxo::{self, UTXO};
use rooch_types::into_address::IntoAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::L1BlockWithBody;
use std::collections::HashMap;
use tracing::{debug, info};

#[tokio::test]
async fn test_submit_block() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    // Mainnet block 00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7
    let block_hex = Vec::<u8>::from_hex("010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b0201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0804ffff001d026e04ffffffff0100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bdef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99c48b0e3f39c51347a19a5819651503a5ac00000000010000000321f75f3139a013f50f315b23b0c9a2b6eac31e2bec98e5891c924664889942260000000049483045022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067fa6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a5014081b5c01ffffffff79cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b84d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea5982f88cef651e9354f5791602369bf5a82a6cd61a62501fffffffffe09f5fe3ffbf5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e82000000004847304402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d33812a5ce01ffffffff01009d966b01000000434104ea1feff861b51fe3f5f8a3b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324bad71f9904ac0ae7336507d785b17a2c115e427a32fac00000000").unwrap();
    let height = 496u64;
    let block: Block = deserialize(&block_hex).unwrap();
    let bitcoin_txdata = block.txdata.clone();
    let block_hash = block.header.block_hash();
    let block_header: Header = block.header.into();
    let move_block = rooch_types::bitcoin::types::Block::from(block.clone());

    binding_test
        .execute_l1_block_and_tx(L1BlockWithBody {
            block: rooch_types::transaction::L1Block {
                chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
                block_height: height,
                block_hash: block_hash.to_byte_array().to_vec(),
            },
            block_body: move_block.encode(),
        })
        .unwrap();

    // let moveos = binding_test.reader_executor.moveos().read();
    let bitcoin_module = binding_test.as_module_binding::<rooch_types::bitcoin::BitcoinModule>();
    assert_eq!(
        bitcoin_module.get_block(block_hash).unwrap().unwrap(),
        block_header
    );

    assert_eq!(
        bitcoin_module.get_block_by_height(height).unwrap().unwrap(),
        block_header
    );
    let latest_block = bitcoin_module.get_latest_block().unwrap().unwrap();
    assert_eq!(height, latest_block.block_height,);
    info!("txdata len: {}", bitcoin_txdata.len());

    check_utxo(bitcoin_txdata, &binding_test);

    let timestamp_module =
        binding_test.as_module_binding::<moveos_types::moveos_std::timestamp::TimestampModule>();

    let now_milliseconds = timestamp_module.now_milliseconds().unwrap();
    let duration = std::time::Duration::from_secs(block_header.time as u64);
    info!(
        "now_milliseconds: {}, header_timestamp: {}",
        now_milliseconds, block_header.time as u64
    );
    assert_eq!(now_milliseconds, duration.as_millis() as u64);
}

fn test_block_process(blocks: Vec<(u64, Block)>) {
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    for (height, block) in blocks {
        let block_hash = block.header.block_hash();
        let move_block = rooch_types::bitcoin::types::Block::from(block.clone());
        binding_test
            .execute_l1_block_and_tx(L1BlockWithBody {
                block: rooch_types::transaction::L1Block {
                    chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
                    block_height: height,
                    block_hash: block_hash.to_byte_array().to_vec(),
                },
                block_body: move_block.encode(),
            })
            .unwrap();
        check_utxo(block.txdata, &binding_test);
    }
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

    let moveos_resolver = binding_test.resolver();

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
        let utxo_object = utxo_state.into_object::<UTXO>().unwrap();
        assert_eq!(utxo_object.value.txid, outpoint.txid);
        assert_eq!(utxo_object.value.vout, outpoint.vout);
        assert_eq!(utxo_object.value.value, tx_out.value.to_sat());
    }

    let ord_module = binding_test.as_module_binding::<rooch_types::bitcoin::ord::OrdModule>();

    let inscriptions = txs
        .iter()
        .flat_map(|tx| {
            let txid = tx.txid();
            let rooch_btc_tx = rooch_types::bitcoin::types::Transaction::from(tx.clone());
            ord_module
                .from_transaction(&rooch_btc_tx, vec![], 0, 0)
                .unwrap()
                .into_iter()
                .enumerate()
                .map(move |(idx, i)| (txid, idx, i))
        })
        .collect::<Vec<_>>();
    for (txid, index, inscription) in inscriptions {
        let txid_address = txid.into_address();
        let index = index as u32;
        let inscription_id = InscriptionID::new(txid_address, index);
        debug!("check inscription: {:?}", inscription_id);

        let object_id = ord::derive_inscription_id(&inscription_id);
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
        let inscription_object = inscription_state.into_object::<Inscription>().unwrap();
        assert_eq!(inscription_object.value.txid, txid.into_address());
        assert_eq!(inscription_object.value.index, index);
        assert_eq!(inscription_object.value.body, inscription.body,);
    }
}

//this test takes too long time in debug mod run it in release mod, use command:
//RUST_LOG=debug cargo test --release --package rooch-framework-tests --lib -- --include-ignored tests::bitcoin_test::test_real_bocks
#[tokio::test]
async fn test_real_bocks() {
    let _ = tracing_subscriber::fmt::try_init();
    if cfg!(debug_assertions) {
        info!("test_real_bocks is ignored in debug mode, please run it in release mode");
        return;
    }
    let cases = vec![
        (Network::Bitcoin, vec![91812u64, 91842u64]),
        (Network::Bitcoin, vec![818677u64]),
        (Network::Testnet, vec![2821527u64]),
    ];
    for (network, heights) in cases {
        info!(
            "test_real_bocks: network: {:?}, height: {:?}",
            network, heights
        );
        let blocks = heights
            .into_iter()
            .map(|height| (height, load_block(network, height)))
            .collect();
        test_block_process(blocks);
    }
}
