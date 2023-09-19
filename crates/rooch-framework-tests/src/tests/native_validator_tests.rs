// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::transaction::MoveAction;
use rooch_key::keypair::KeyPairType;
use rooch_key::keystore::{AccountKeystore, InMemKeystore};
use rooch_types::address::RoochAddress;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::framework::empty::Empty;
use rooch_types::transaction::{rooch::RoochTransactionData, AbstractTransaction};

use crate::binding_test;

#[test]
fn test_validate() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let native_validator = binding_test
        .as_module_bundle::<rooch_types::framework::native_validator::NativeValidatorModule>(
    );

    let keystore = InMemKeystore::<RoochAddress, RoochKeyPair>::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(&sender, tx_data, KeyPairType::RoochKeyPairType)
        .unwrap();
    let auth_info = tx.authenticator_info().unwrap();
    let move_tx = tx.construct_moveos_transaction(sender.into()).unwrap();

    native_validator
        .validate(&move_tx.ctx, auth_info.authenticator.payload)
        .unwrap();
}
