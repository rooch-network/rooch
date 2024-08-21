// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use rooch_types::bitcoin::ord::{Inscription, InscriptionID};
use tracing::{debug, warn};

use crate::bitcoin_block_tester::BitcoinBlockTester;

// This test for testing BitcoinBlockTester
#[tokio::test]
async fn test_block_100000() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut tester = BitcoinBlockTester::new(100000).unwrap();
    tester.execute().unwrap();
    tester.verify_utxo().unwrap();
    tester.verify_inscriptions().unwrap();
}

#[tokio::test]
async fn test_block_790964() {
    let _ = tracing_subscriber::fmt::try_init();

    if cfg!(debug_assertions) {
        warn!("test_block_790964 is ignored in debug mode, please run it in release mode");
        return;
    }

    let mut tester = BitcoinBlockTester::new(790964).unwrap();
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
    assert_eq!(inscription.offset, 0u64);
}
