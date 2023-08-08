// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::{module_binding::ModuleBundle, transaction::MoveAction};
use rooch_framework::{bindings::empty::Empty, ROOCH_FRAMEWORK_ADDRESS};
use rooch_key::keystore::{AccountKeystore, InMemKeystore};
use rooch_types::{
    crypto::BuiltinScheme,
    framework::session_key::SessionScope,
    transaction::{rooch::RoochTransactionData, AbstractTransaction},
};

use crate::binding_test;

#[test]
fn test_validate_ed25519() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test.as_module_bundle::<rooch_framework::bindings::transaction_validator::TransactionValidator>();

    let keystore = InMemKeystore::new_ed25519_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(&sender, tx_data, BuiltinScheme::Ed25519)
        .unwrap();
    let auth_info = tx.authenticator_info();
    let move_tx = tx.construct_moveos_transaction(sender.into()).unwrap();

    transaction_validator
        .validate(&move_tx.ctx, auth_info)
        .unwrap();
}

#[test]
fn test_validate_ecdsa() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test.as_module_bundle::<rooch_framework::bindings::transaction_validator::TransactionValidator>();

    let keystore = InMemKeystore::new_ecdsa_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(&sender, tx_data, BuiltinScheme::Ecdsa)
        .unwrap();
    let auth_info = tx.authenticator_info();
    let move_tx = tx.construct_moveos_transaction(sender.into()).unwrap();

    transaction_validator
        .validate(&move_tx.ctx, auth_info)
        .unwrap();
}

#[test]
fn test_validate_ecdsa_recoverable() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test.as_module_bundle::<rooch_framework::bindings::transaction_validator::TransactionValidator>();

    let keystore = InMemKeystore::new_ecdsa_recoverable_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(&sender, tx_data, BuiltinScheme::EcdsaRecoverable)
        .unwrap();
    let auth_info = tx.authenticator_info();
    let move_tx = tx.construct_moveos_transaction(sender.into()).unwrap();

    transaction_validator
        .validate(&move_tx.ctx, auth_info)
        .unwrap();
}

#[test]
fn test_validate_schnorr() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test.as_module_bundle::<rooch_framework::bindings::transaction_validator::TransactionValidator>();

    let keystore = InMemKeystore::new_schnorr_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(&sender, tx_data, BuiltinScheme::Schnorr)
        .unwrap();
    let auth_info = tx.authenticator_info();
    let move_tx = tx.construct_moveos_transaction(sender.into()).unwrap();

    transaction_validator
        .validate(&move_tx.ctx, auth_info)
        .unwrap();
}

#[test]
fn test_session_key_ed25519() {
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let keystore = InMemKeystore::new_ed25519_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    let auth_key = vec![1u8; 32];
    let session_scope = SessionScope::new(
        ROOCH_FRAMEWORK_ADDRESS,
        Empty::MODULE_NAME.as_str(),
        Empty::EMPTY_FUNCTION_NAME.as_str(),
    );
    let expiration_time = 100;
    let max_inactive_interval = 100;
    let action =
        rooch_framework::bindings::session_key::SessionKeyModule::create_session_key_action(
            auth_key.clone(),
            BuiltinScheme::Ed25519,
            session_scope.clone(),
            expiration_time,
            max_inactive_interval,
        );
    let tx_data = RoochTransactionData::new(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(&sender, tx_data, BuiltinScheme::Ed25519)
        .unwrap();
    binding_test.execute(tx).unwrap();

    let session_key_module =
        binding_test.as_module_bundle::<rooch_framework::bindings::session_key::SessionKeyModule>();
    let session_key_option = session_key_module
        .get_session_key(sender.into(), auth_key)
        .unwrap();
    assert!(session_key_option.is_some(), "Session key not found");
    let session_key = session_key_option.unwrap();
    assert_eq!(session_key.scheme, BuiltinScheme::Ed25519.flag() as u64);
    assert_eq!(session_key.scopes, vec![session_scope]);
    assert_eq!(session_key.expiration_time, expiration_time);
    assert_eq!(session_key.max_inactive_interval, max_inactive_interval);
}
