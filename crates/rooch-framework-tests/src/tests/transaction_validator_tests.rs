// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveValue;
use moveos_types::move_types::FunctionId;
use moveos_types::{module_binding::ModuleBinding, transaction::MoveAction};
use rooch_key::keystore::{AccountKeystore, InMemKeystore};
use rooch_types::{addresses::ROOCH_FRAMEWORK_ADDRESS, framework::empty::Empty};
use rooch_types::{
    crypto::BuiltinScheme,
    framework::session_key::SessionScope,
    transaction::{rooch::RoochTransactionData, AbstractTransaction},
};

use crate::binding_test;

#[test]
fn test_validate_ed25519() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test
        .as_module_bundle::<rooch_types::framework::transaction_validator::TransactionValidator>(
    );

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
    let transaction_validator = binding_test
        .as_module_bundle::<rooch_types::framework::transaction_validator::TransactionValidator>(
    );

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
    let transaction_validator = binding_test
        .as_module_bundle::<rooch_types::framework::transaction_validator::TransactionValidator>(
    );

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
    let transaction_validator = binding_test
        .as_module_bundle::<rooch_types::framework::transaction_validator::TransactionValidator>(
    );

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
    tracing_subscriber::fmt::init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let mut keystore = InMemKeystore::new_ed25519_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    let session_auth_key = keystore.generate_session_key(&sender).unwrap();

    let session_scope = SessionScope::new(
        ROOCH_FRAMEWORK_ADDRESS,
        Empty::MODULE_NAME.as_str(),
        Empty::EMPTY_FUNCTION_NAME.as_str(),
    );
    let expiration_time = 100;
    let max_inactive_interval = 100;
    let action = rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
        session_auth_key.as_ref().to_vec(),
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
        binding_test.as_module_bundle::<rooch_types::framework::session_key::SessionKeyModule>();
    let session_key_option = session_key_module
        .get_session_key(sender.into(), &session_auth_key)
        .unwrap();
    assert!(session_key_option.is_some(), "Session key not found");
    let session_key = session_key_option.unwrap();
    assert_eq!(&session_key.authentication_key, session_auth_key.as_ref());
    assert_eq!(session_key.scopes, vec![session_scope]);
    assert_eq!(session_key.expiration_time, expiration_time);
    assert_eq!(session_key.max_inactive_interval, max_inactive_interval);

    // send transaction via session key

    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new(sender, sequence_number + 1, action);
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key)
        .unwrap();

    binding_test.execute(tx).unwrap();

    // test the session key call function is out the scope.

    let action = MoveAction::new_function_call(
        FunctionId::new(
            ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, ident_str!("account").to_owned()),
            ident_str!("create_account_entry").to_owned(),
        ),
        vec![],
        vec![MoveValue::Address(AccountAddress::random())
            .simple_serialize()
            .unwrap()],
    );
    let tx_data = RoochTransactionData::new(sender, sequence_number + 2, action);
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key)
        .unwrap();

    // the session key is not in the scope of account module, so the transaction should be rejected when validate.
    // TODO Get the validate VMStatus and check the error code.
    let execute_result = binding_test.execute_as_result(tx);
    assert!(execute_result.is_err(), "expect move abort");
    //let result = binding_test.execute_as_result(tx).unwrap();
    // match result.transaction_info.status {
    //     KeptVMStatus::MoveAbort(l, code) => {
    //         match l{
    //             AbortLocation::Module(module_id) => {
    //                 assert_eq!(module_id, ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, ident_str!("session_key").to_owned()), "expect session key module");
    //             }
    //             _ => panic!("expect move abort in module"),
    //         }
    //         let (_category, reason) = error::explain(code);
    //         // EFunctionCallBeyoundSessionScope = 5
    //         assert_eq!(reason, 5, "expect EFunctionCallBeyoundSessionScope");
    //     }
    //     _ => panic!("expect move abort"),
    // }
}
