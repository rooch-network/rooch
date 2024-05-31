// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use ethers::prelude::*;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::transaction::MoveAction;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::framework::ethereum::BlockHeader;
use rooch_types::transaction::rooch::RoochTransactionData;

#[test]
fn test_submit_block() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    let json = serde_json::json!(
    {
        "baseFeePerGas": "0x7",
        "miner": "0x0000000000000000000000000000000000000001",
        "number": "0x1b4",
        "hash": "0x0e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331",
        "parentHash": "0x9646252be9520f6e71339a8df9c55e4d7619deeb018d2a3f2d21fc165dde5eb5",
        "mixHash": "0x1010101010101010101010101010101010101010101010101010101010101010",
        "nonce": "0x0000000000000000",
        "sealFields": [
          "0xe04d296d2460cfb8472af2c5fd05b5a214109c25688d3704aed5484f9a7792f2",
          "0x0000000000000042"
        ],
        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "logsBloom":  "0x0e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331",
        "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        "stateRoot": "0xd5855eb08b3387c0af375e9cdb6acfc05eb8f519e419b874b6ff2ffda7ed1dff",
        "difficulty": "0x27f07",
        "totalDifficulty": "0x27f07",
        "extraData": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "size": "0x27f07",
        "gasLimit": "0x9f759",
        "minGasPrice": "0x9f759",
        "gasUsed": "0x9f759",
        "timestamp": "0x54e34e8e",
        "transactions": [],
        "uncles": []
      }
    );

    let ethereum_block: Block<()> = serde_json::from_value(json).unwrap();

    let block_header = BlockHeader::try_from(&ethereum_block).unwrap();
    let action = MoveAction::Function(
        rooch_types::framework::ethereum::EthereumModule::create_submit_new_block_call(
            &block_header,
        ),
    );
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let timestamp_module =
        binding_test.as_module_binding::<moveos_types::moveos_std::timestamp::TimestampModule>();

    let now_milliseconds = timestamp_module.now_milliseconds().unwrap();
    let duration = std::time::Duration::from_secs(block_header.timestamp.unchecked_as_u64());
    println!(
        "now_milliseconds: {}, header_timestamp: {}",
        now_milliseconds, block_header.timestamp
    );
    assert_eq!(now_milliseconds, duration.as_millis() as u64);
}
