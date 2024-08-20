// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
