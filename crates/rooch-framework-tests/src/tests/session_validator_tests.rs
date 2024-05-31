// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::{AbortLocation, KeptVMStatus, VMStatus};
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_std::string::MoveString;
use moveos_types::move_types::FunctionId;
use moveos_types::{module_binding::ModuleBinding, transaction::MoveAction};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::framework::session_key::SessionKeyModule;
use rooch_types::framework::session_validator::SessionValidatorModule;
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

    let session_auth_key = keystore.generate_session_key(&sender, None).unwrap();

    let session_scope = SessionScope::new(ROOCH_FRAMEWORK_ADDRESS, "*", "*").unwrap();
    let app_name = MoveString::from_str("test").unwrap();
    let app_url = MoveString::from_str("https:://test.rooch.network").unwrap();
    let max_inactive_interval = 100;
    let action = rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
        app_name.clone(),
        app_url.clone(),
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
    assert_eq!(session_key.scopes, vec![session_scope.clone()]);
    assert_eq!(session_key.max_inactive_interval, max_inactive_interval);
    keystore.binding_session_key(sender, session_key).unwrap();

    // send transaction via session key, it in the scop of session key, so it should success.

    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 1, action);
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key, None)
        .unwrap();

    binding_test.execute(tx).unwrap();

    // test the session key call function which not in the scope of session key, it should be rejected.

    let action = MoveAction::new_function_call(
        FunctionId::new(
            ModuleId::new(AccountAddress::random(), ident_str!("empty").to_owned()),
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
                        &SessionValidatorModule::module_id(),
                        "expect session validator module"
                    );
                }
                _ => panic!("expect move abort in module"),
            }
            // ErrorFunctionCallBeyondSessionScope = 3
            assert_eq!(*code, 3, "expect ErrorFunctionCallBeyondSessionScope");
        }
        _ => {
            panic!("Expect move abort")
        }
    }

    // test create session key with session key, it should fail.

    let new_session_auth_key = keystore.generate_session_key(&sender, None).unwrap();
    let action = rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
        app_name,
        app_url,
        new_session_auth_key.as_ref().to_vec(),
        session_scope.clone(),
        max_inactive_interval,
    );
    // because previous transaction is failed, so the sequence number is not increased.
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 2, action);
    // sign with the old session key
    let tx = keystore
        .sign_transaction_via_session_key(&sender, tx_data, &session_auth_key, None)
        .unwrap();

    let execute_result = binding_test.execute_as_result(tx).unwrap();
    match execute_result.output.status {
        KeptVMStatus::MoveAbort(l, code) => {
            match l {
                AbortLocation::Module(module_id) => {
                    assert_eq!(
                        module_id,
                        SessionKeyModule::module_id(),
                        "expect session key module"
                    );
                }
                _ => panic!("expect move abort in module"),
            }
            // ErrorSessionKeyCreatePermissionDenied = 1
            assert_eq!(code, 1, "expect ErrorSessionKeyCreatePermissionDenied");
        }
        _ => {
            panic!("Expect move abort")
        }
    }

    // test session key expired
    let update_time_action =
        TimestampModule::create_fast_forward_seconds_for_local_action(max_inactive_interval + 1);

    let tx_data =
        RoochTransactionData::new_for_test(sender, sequence_number + 3, update_time_action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();
    binding_test.execute(tx).unwrap();

    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number + 4, action);
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
                        &SessionValidatorModule::module_id(),
                        "expect session key module"
                    );
                }
                _ => panic!("expect move abort in module"),
            }
            // ErrorSessionIsExpired = 2
            assert_eq!(*code, 2, "expect ErrorSessionIsExpired");
        }
        _ => {
            panic!("Expect move abort")
        }
    }
}
