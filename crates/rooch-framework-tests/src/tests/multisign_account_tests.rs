// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::nursery::multisign_account::{self, MultisignAccountModule};

#[tokio::test]
async fn test_multisign_account() {
    let _ = tracing_subscriber::fmt::try_init();
    let binding_test = binding_test::RustBindingTest::new().unwrap();

    let keystore = InMemKeystore::new_insecure_for_tests(3);

    let u1 = keystore.addresses()[0];
    let u2 = keystore.addresses()[1];
    let u3 = keystore.addresses()[2];

    // println!("u1: {:?}", u1);
    // println!("u2: {:?}", u2);
    // println!("u3: {:?}", u3);

    let kp1 = keystore.get_key_pair(&u1, None).unwrap();
    let kp2 = keystore.get_key_pair(&u2, None).unwrap();
    let kp3 = keystore.get_key_pair(&u3, None).unwrap();

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

    let account_module = binding_test.as_module_binding::<MultisignAccountModule>();

    let bitcoin_address_from_move = account_module
        .generate_multisign_address(2, pubkeys)
        .unwrap();

    assert_eq!(bitcoin_address_from_rust, bitcoin_address_from_move);
}
