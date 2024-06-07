// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoin::consensus::deserialize;
use bitcoin::hashes::Hash;
use bitcoin::hex::FromHex;
use bitcoincore_rpc_json::bitcoin;
use bitcoincore_rpc_json::bitcoin::Block;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_store::RoochStore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::L1BlockWithBody;
use std::fs;

use crate::config::TxType;
use crate::tx::TxType::{Empty, Transfer};

pub const EXAMPLE_SIMPLE_BLOG_PACKAGE_NAME: &str = "simple_blog";
pub const EXAMPLE_SIMPLE_BLOG_NAMED_ADDRESS: &str = "simple_blog";

pub fn gen_sequencer(keypair: RoochKeyPair, rooch_store: RoochStore) -> Result<SequencerActor> {
    SequencerActor::new(keypair, rooch_store.clone())
}

pub fn create_publish_transaction(
    test_transaction_builder: &TestTransactionBuilder,
    keystore: &InMemKeystore,
) -> Result<RoochTransaction> {
    let publish_action = test_transaction_builder.new_publish_examples(
        EXAMPLE_SIMPLE_BLOG_PACKAGE_NAME,
        Some(EXAMPLE_SIMPLE_BLOG_NAMED_ADDRESS.to_string()),
    )?;
    let tx_data = test_transaction_builder.build(publish_action);
    let rooch_tx =
        keystore.sign_transaction(&test_transaction_builder.sender.into(), tx_data, None)?;
    Ok(rooch_tx)
}

pub fn create_l2_tx(
    test_transaction_builder: &mut TestTransactionBuilder,
    keystore: &InMemKeystore,
    seq_num: u64,
    tx_type: TxType,
) -> Result<RoochTransaction> {
    test_transaction_builder.update_sequence_number(seq_num);

    let action = match tx_type {
        Empty => test_transaction_builder.call_empty_create(),
        Transfer => test_transaction_builder.call_transfer_create(),
        _ => panic!("Unsupported tx type"),
    };

    let tx_data = test_transaction_builder.build(action);
    let rooch_tx =
        keystore.sign_transaction(&test_transaction_builder.sender.into(), tx_data, None)?;
    Ok(rooch_tx)
}

pub fn find_block_height(dir: String) -> Result<Vec<u64>> {
    let mut block_heights = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "hex" {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let height: u64 = file_stem
                .parse()
                .expect("Failed to parse block height from filename");
            block_heights.push(height);
        }
    }

    block_heights.sort();
    Ok(block_heights)
}

pub fn create_btc_blk_tx(height: u64, block_file: String) -> Result<L1BlockWithBody> {
    let block_hex_str = fs::read_to_string(block_file).unwrap();
    let block_hex = Vec::<u8>::from_hex(&block_hex_str).unwrap();
    let origin_block: Block = deserialize(&block_hex).unwrap();
    let block = origin_block.clone();
    let block_hash = block.header.block_hash();
    let move_block = rooch_types::bitcoin::types::Block::from(block.clone());
    Ok(L1BlockWithBody {
        block: rooch_types::transaction::L1Block {
            chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
            block_height: height,
            block_hash: block_hash.to_byte_array().to_vec(),
        },
        block_body: move_block.encode(),
    })
}
