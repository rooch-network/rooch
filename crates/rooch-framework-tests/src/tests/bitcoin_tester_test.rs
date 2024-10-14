// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::bitcoin_block_tester::BitcoinBlockTester;
use rooch_types::bitcoin::ord::{Inscription, InscriptionID, SatPoint};
use std::str::FromStr;
use tracing::{debug, warn};

// This test for testing BitcoinBlockTester
#[tokio::test]
async fn test_block_100000() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut tester = BitcoinBlockTester::new(100000).unwrap();
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();
}

// Some testcase in the issue
// https://github.com/rooch-network/rooch/issues/1985

// cargo run -p rooch-framework-tests --  --btc-rpc-url http://localhost:8332 --btc-rpc-username your_username --btc-rpc-password your_pwd --blocks 790964 --blocks 855396
// This test contains two block: 790964 and 855396
// The inscription 8706753 inscribed in block 790964 and spend as fee in block 855396
// https://ordiscan.com/inscription/8706753
// https://ordinals.com/inscription/8706753
#[tokio::test]
async fn test_block_790964() {
    let _ = tracing_subscriber::fmt::try_init();

    if cfg!(debug_assertions) {
        warn!("test_block_790964 is ignored in debug mode, please run it in release mode");
        return;
    }

    let mut tester = BitcoinBlockTester::new(790964).unwrap();
    //Execute the first block 790964
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();
    let inscription_id = InscriptionID::from_str(
        "4b8111663106c242da8580ba38c36f261287b9c35b1aa5974f4c14306905e720i0",
    )
    .unwrap();
    let inscription_opt = tester.get_inscription(&inscription_id).unwrap();
    assert!(inscription_opt.is_some());
    let inscription_obj = inscription_opt.unwrap();
    let inscription = inscription_obj.value_as::<Inscription>().unwrap();
    debug!("Inscription: {:?}", inscription);
    assert_eq!(inscription.id(), inscription_id);
    // Because we do not execute bitcoin block from ordinals genesis, so the inscription number and sequence number are not the same as the real inscription.
    // let expected_inscription_number = 8706753u32;
    // let expected_sequence_number = 8709019u32;
    // assert_eq!(inscription.inscription_number, expected_inscription_number);
    // assert_eq!(inscription.sequence_number, expected_sequence_number);
    assert_eq!(inscription.location.offset, 0u64);

    //Execute the second block 855396
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();

    let inscription_obj = tester.get_inscription(&inscription_id).unwrap().unwrap();
    let inscription = inscription_obj.value_as::<Inscription>().unwrap();
    debug!("Inscription: {:?}", inscription);
    assert_eq!(
        inscription.location,
        SatPoint::from_str(
            "4a61ddc33e4a0b99fa69aac4d2d5de9efe7c7cc44d5d28a9ac1734f8c3317964:0:316084756"
        )
        .unwrap()
    );
}

#[tokio::test]
async fn test_block_781735() {
    let _ = tracing_subscriber::fmt::try_init();

    if cfg!(debug_assertions) {
        warn!("test_block_781735 is ignored in debug mode, please run it in release mode");
        return;
    }

    let mut tester = BitcoinBlockTester::new(781735).unwrap();
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();

    //https://ordiscan.com/inscription/-453
    //is a curse inscription
    let inscription_id = InscriptionID::from_str(
        "092111e882a8025f3f05ab791982e8cc7fd7395afe849a5949fd56255b5c41cci24",
    )
    .unwrap();

    let inscription_opt = tester.get_inscription(&inscription_id).unwrap();
    assert!(inscription_opt.is_some());
    let inscription_obj = inscription_opt.unwrap();
    let inscription = inscription_obj.value_as::<Inscription>().unwrap();
    debug!("Inscription: {:?}", inscription);
    assert_eq!(inscription.id(), inscription_id);
    assert!(inscription.is_cursed);
}

//Inscription use pointer to set the offset, and mint multi inscription in one input.
//https://ordinals.com/tx/6ea3bf728b34c8c01ba4703e00ad688be100599b92fbdac71e6aea6ad8355552
#[tokio::test]
async fn test_block_832918() {
    let _ = tracing_subscriber::fmt::try_init();

    if cfg!(debug_assertions) {
        warn!("test_block_781735 is ignored in debug mode, please run it in release mode");
        return;
    }

    let mut tester = BitcoinBlockTester::new(832918).unwrap();
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();
}

// Inscription inscribe and transfer in same tx
// https://ordinals.com/tx/207322afdcca902cb36aeb674214dc5f80f9593f12c1de57830ad33adae46a0a
#[tokio::test]
async fn test_block_794970() {
    let _ = tracing_subscriber::fmt::try_init();

    if cfg!(debug_assertions) {
        warn!("test_block_781735 is ignored in debug mode, please run it in release mode");
        return;
    }

    let mut tester = BitcoinBlockTester::new(794970).unwrap();
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();
}
