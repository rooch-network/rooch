// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use moveos_types::module_binding::MoveFunctionCaller;
use rand::Rng;
use rooch_types::{
    bitcoin::multisign_account::{self, MultisignAccountModule},
    crypto::RoochKeyPair,
    genesis_config::G_TEST_CONFIG,
    transaction::RoochTransactionData,
};

#[tokio::test]
async fn test_multisign_account() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let kp1 = RoochKeyPair::generate_secp256k1();
    let kp2 = RoochKeyPair::generate_secp256k1();
    let kp3 = RoochKeyPair::generate_secp256k1();

    let u1 = kp1.public().bitcoin_address().unwrap().to_rooch_address();
    let u2 = kp2.public().bitcoin_address().unwrap().to_rooch_address();
    let u3 = kp3.public().bitcoin_address().unwrap().to_rooch_address();

    // println!("u1: {:?}", u1);
    // println!("u2: {:?}", u2);
    // println!("u3: {:?}", u3);

    let pubkeys = vec![
        kp1.bitcoin_public_key().unwrap(),
        kp2.bitcoin_public_key().unwrap(),
        kp3.bitcoin_public_key().unwrap(),
    ];
    // for pubkey in &pubkeys {
    //     println!("pubkey: {}", pubkey);
    // }

    let pubkeys = pubkeys
        .into_iter()
        .map(|pk| pk.to_bytes())
        .collect::<Vec<_>>();

    let bitcoin_address_from_rust =
        multisign_account::generate_multisign_address(2, pubkeys.clone()).unwrap();
    //println!("bitcoin_address_from_rust: {}", bitcoin_address_from_rust);

    let multisign_address = {
        let account_module = binding_test.as_module_binding::<MultisignAccountModule>();

        let bitcoin_address_from_move = account_module
            .generate_multisign_address(2, pubkeys.clone())
            .unwrap();

        assert_eq!(bitcoin_address_from_rust, bitcoin_address_from_move);
        bitcoin_address_from_move.to_rooch_address()
    };

    let action = MultisignAccountModule::initialize_multisig_account_action(2, pubkeys);
    let tx_data = RoochTransactionData::new_for_test(u1, 0, action);
    let tx = tx_data.sign(&kp1);
    binding_test.execute(tx).unwrap();

    let account_module = binding_test.as_module_binding::<MultisignAccountModule>();
    assert!(account_module
        .is_participant(multisign_address.into(), u1.into())
        .unwrap());
    assert!(account_module
        .is_participant(multisign_address.into(), u2.into())
        .unwrap());
    assert!(account_module
        .is_participant(multisign_address.into(), u3.into())
        .unwrap());
}

#[tokio::test]
async fn test_genesis_multisign_account() {
    let _ = tracing_subscriber::fmt::try_init();
    let binding_test = binding_test::RustBindingTest::new().unwrap();

    let multisign_config = G_TEST_CONFIG.rooch_dao.clone();
    let genesis_multisign_account = multisign_account::generate_multisign_address(
        multisign_config.threshold as usize,
        multisign_config.participant_public_keys.clone(),
    )
    .unwrap();

    assert_eq!(
        multisign_config.multisign_bitcoin_address,
        genesis_multisign_account
    );

    let account_module = binding_test.as_module_binding::<MultisignAccountModule>();

    let genesis_multisign_account_from_move = account_module
        .generate_multisign_address(
            multisign_config.threshold,
            multisign_config.participant_public_keys.clone(),
        )
        .unwrap();

    assert_eq!(
        genesis_multisign_account,
        genesis_multisign_account_from_move
    );
}

#[tokio::test]
async fn test_multisign_account_random() {
    let _ = tracing_subscriber::fmt::try_init();
    let binding_test = binding_test::RustBindingTest::new().unwrap();

    let count = rand::thread_rng().gen_range(3..10);
    let mut pubkeys = Vec::with_capacity(count);
    for _ in 0..count {
        let kp = RoochKeyPair::generate_secp256k1();
        pubkeys.push(kp.bitcoin_public_key().unwrap().to_bytes());
    }
    let threshold = rand::thread_rng().gen_range(2..count);
    let bitcoin_address_from_rust =
        multisign_account::generate_multisign_address(threshold, pubkeys.clone()).unwrap();

    let account_module = binding_test.as_module_binding::<MultisignAccountModule>();
    let bitcoin_address_from_move = account_module
        .generate_multisign_address(threshold as u64, pubkeys.clone())
        .unwrap();

    assert_eq!(
        bitcoin_address_from_rust,
        bitcoin_address_from_move,
        "threshold: {}, count: {}, pubkeys:{}",
        threshold,
        count,
        pubkeys
            .iter()
            .map(hex::encode)
            .collect::<Vec<_>>()
            .join(",")
    );
}
