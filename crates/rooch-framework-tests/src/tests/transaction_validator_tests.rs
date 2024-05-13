// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::{AbortLocation, VMStatus};
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_std::ascii::MoveAsciiString;
use moveos_types::move_std::string::MoveString;
use moveos_types::move_types::FunctionId;
use moveos_types::{module_binding::ModuleBinding, transaction::MoveAction};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::framework::session_key::SessionKeyModule;
use rooch_types::framework::timestamp::TimestampModule;
use rooch_types::{addresses::ROOCH_FRAMEWORK_ADDRESS, framework::empty::Empty};
use rooch_types::{framework::session_key::SessionScope, transaction::rooch::RoochTransactionData};
use std::str::FromStr;

#[test]
fn test_session_key_rooch() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let mut keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    let session_auth_key = keystore.generate_session_key(&sender, None, None).unwrap();

    let session_scope = SessionScope::new(
        ROOCH_FRAMEWORK_ADDRESS,
        Empty::MODULE_NAME.as_str(),
        Empty::EMPTY_FUNCTION_NAME.as_str(),
    )
    .unwrap();
    let app_name = MoveString::from_str("test").unwrap();
    let app_url = MoveAsciiString::from_str("https://test-seed.rooch.network").unwrap();
    let max_inactive_interval = 100;
    let action = rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
        app_name,
        app_url,
        session_auth_key.as_ref().to_vec(),
        session_scope.clone(),
        max_inactive_interval,
    );
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let session_key_module =
        binding_test.as_module_binding::<rooch_types::framework::session_key::SessionKeyModule>();
    let session_key_option = session_key_module
        .get_session_key(sender.into(), &session_auth_key)
        .unwrap();
    assert!(session_key_option.is_some(), "Session key not found");
    let session_key = session_key_option.unwrap();
    assert_eq!(&session_key.authentication_key, session_auth_key.as_ref());
    assert_eq!(session_key.scopes, vec![session_scope]);
    assert_eq!(session_key.max_inactive_interval, max_inactive_interval);
    keystore.binding_session_key(sender, session_key).unwrap();

    // send transaction via session key

    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 1, action);
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key, None)
        .unwrap();

    binding_test.execute(tx).unwrap();

    // test the session key call function is out the scope.

    let action = MoveAction::new_function_call(
        FunctionId::new(
            ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, ident_str!("empty").to_owned()),
            ident_str!("empty_with_signer").to_owned(),
        ),
        vec![],
        vec![MoveValue::Address(AccountAddress::random())
            .simple_serialize()
            .unwrap()],
    );
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 2, action);
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key, None)
        .unwrap();

    // the session key is not in the scope of account module, so the transaction should be rejected when validate.
    let execute_result = binding_test.execute_as_result(tx);
    let error = execute_result.expect_err("expect transaction validate error");
    match error.downcast_ref() {
        Some(VMStatus::MoveAbort(l, code)) => {
            match l {
                AbortLocation::Module(module_id) => {
                    assert_eq!(
                        module_id,
                        &SessionKeyModule::module_id(),
                        "expect session key module"
                    );
                }
                _ => panic!("expect move abort in module"),
            }
            // ErrorFunctionCallBeyondSessionScope = 5
            assert_eq!(*code, 5, "expect ErrorFunctionCallBeyondSessionScope");
        }
        _ => {
            panic!("Expect move abort")
        }
    }

    // test session key expired
    let update_time_action =
        TimestampModule::create_fast_forward_seconds_for_local_action(max_inactive_interval + 1);
    // because previous transaction is failed, so the sequence number is not increased.
    let tx_data =
        RoochTransactionData::new_for_test(sender, sequence_number + 2, update_time_action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 3, action);
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key, None)
        .unwrap();
    let error = binding_test.execute_as_result(tx).unwrap_err();
    match error.downcast_ref() {
        Some(VMStatus::MoveAbort(l, code)) => {
            match l {
                AbortLocation::Module(module_id) => {
                    assert_eq!(
                        module_id,
                        &SessionKeyModule::module_id(),
                        "expect session key module"
                    );
                }
                _ => panic!("expect move abort in module"),
            }
            // ErrorSessionIsExpired = 4
            assert_eq!(*code, 4, "expect ErrorSessionIsExpired");
        }
        _ => {
            panic!("Expect move abort")
        }
    }
}

#[test]
fn test_session_key_from_session_key_failure_rooch() {
    // prepare test
    let _ = tracing_subscriber::fmt::try_init();
    let mut first_binding_test = binding_test::RustBindingTest::new().unwrap();

    let mut keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    // create first session key
    let first_session_auth_key = keystore.generate_session_key(&sender, None, None).unwrap();

    let session_scope_clone = SessionScope::new(
        ROOCH_FRAMEWORK_ADDRESS,
        Empty::MODULE_NAME.as_str(),
        Empty::EMPTY_FUNCTION_NAME.as_str(),
    )
    .unwrap();
    let test_app_name = MoveString::from_str("test").unwrap();
    let test_app_url = MoveAsciiString::from_str("https://test-seed.rooch.network").unwrap();
    let max_inactive_interval = 100;
    let test_action =
        rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
            test_app_name,
            test_app_url,
            first_session_auth_key.as_ref().to_vec(),
            session_scope_clone.clone(),
            max_inactive_interval,
        );
    let test_tx_data = RoochTransactionData::new_for_test(sender, sequence_number, test_action);
    let test_tx = keystore
        .sign_transaction(&sender, test_tx_data, None)
        .unwrap();
    first_binding_test.execute(test_tx).unwrap();

    // bind first session key
    let test_session_key_module = first_binding_test
        .as_module_binding::<rooch_types::framework::session_key::SessionKeyModule>(
    );
    let test_session_key_option = test_session_key_module
        .get_session_key(sender.into(), &first_session_auth_key)
        .unwrap();
    assert!(
        test_session_key_option.is_some(),
        "Test session key not found"
    );
    let test_session_key_clone = test_session_key_option.clone().unwrap();
    assert_eq!(
        &test_session_key_clone.authentication_key,
        first_session_auth_key.as_ref()
    );
    assert_eq!(test_session_key_clone.scopes, vec![session_scope_clone]);
    assert_eq!(
        test_session_key_clone.max_inactive_interval,
        max_inactive_interval
    );
    keystore
        .binding_session_key(sender, test_session_key_clone)
        .unwrap();

    // send transaction via first session key
    let test_action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let test_tx_data =
        RoochTransactionData::new_for_test(sender, sequence_number + 1, test_action.clone());
    let test_tx = keystore
        .sign_transaction_via_session_key(
            &sender,
            test_tx_data.clone(),
            &first_session_auth_key,
            None,
        )
        .unwrap();
    first_binding_test.execute(test_tx).unwrap();

    // create second session key from first session key
    let second_session_auth_key = keystore
        .generate_session_key(&sender, None, test_session_key_option)
        .unwrap();

    let session_scope = SessionScope::new(
        ROOCH_FRAMEWORK_ADDRESS,
        Empty::MODULE_NAME.as_str(),
        Empty::EMPTY_FUNCTION_NAME.as_str(),
    )
    .unwrap();
    let dev_app_name = MoveString::from_str("dev").unwrap();
    let dev_app_url = MoveAsciiString::from_str("https://dev-seed.rooch.network").unwrap();
    let dev_action =
        rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
            dev_app_name,
            dev_app_url,
            second_session_auth_key.as_ref().to_vec(),
            session_scope.clone(),
            max_inactive_interval,
        );
    let dev_tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 2, dev_action);
    let dev_tx = keystore
        .sign_transaction(&sender, dev_tx_data, None)
        .unwrap();
    first_binding_test.execute(dev_tx).unwrap();

    // let dev_session_key_module =
    //     second_binding_test.as_module_binding::<rooch_types::framework::session_key::SessionKeyModule>();
    // let dev_session_key_option = dev_session_key_module
    //     .get_session_key(sender.into(), &second_session_auth_key)
    //     .unwrap();
    // assert!(dev_session_key_option.is_some(), "Dev session key not found");
    // let dev_session_key = dev_session_key_option.unwrap();
    // assert_eq!(&dev_session_key.authentication_key, second_session_auth_key.as_ref());
    // assert_eq!(dev_session_key.scopes, vec![session_scope]);
    // assert_eq!(dev_session_key.max_inactive_interval, max_inactive_interval);
    // keystore.binding_session_key(sender, dev_session_key).unwrap();

    // send transaction via second session key
    // let dev_action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    // let test_tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 2, test_action.clone());
    // let dev_tx = keystore
    //     .sign_transaction_via_session_key(&sender, test_tx_data.clone(), &second_session_auth_key, None)
    //     .unwrap();
    // first_binding_test.execute(dev_tx).unwrap();
}
