// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{Bytes, U256};
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::{AbortLocation, VMStatus};
use moveos_types::move_types::FunctionId;
use moveos_types::{module_binding::ModuleBinding, transaction::MoveAction};
use rooch_key::keystore::{AccountKeystore, InMemKeystore};
use rooch_types::address::{EthereumAddress, MultiChainAddress, RoochAddress};
use rooch_types::framework::session_key::SessionKeyModule;
use rooch_types::framework::timestamp::TimestampModule;
use rooch_types::keypair_type::KeyPairType;
use rooch_types::transaction::ethereum::EthereumTransactionData;
use rooch_types::{addresses::ROOCH_FRAMEWORK_ADDRESS, framework::empty::Empty};
use rooch_types::{
    framework::session_key::SessionScope,
    transaction::{rooch::RoochTransactionData, AbstractTransaction},
};

use crate::binding_test;

#[test]
fn test_validate_rooch() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test
        .as_module_bundle::<rooch_types::framework::transaction_validator::TransactionValidator>(
    );

    let keystore = InMemKeystore::<RoochAddress>::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(
            &sender,
            tx_data,
            KeyPairType::RoochKeyPairType,
            Some("".to_owned()),
        )
        .unwrap();
    let auth_info = tx.authenticator_info().unwrap();
    let move_tx = tx.construct_moveos_transaction(sender.into()).unwrap();

    transaction_validator
        .validate(&move_tx.ctx, auth_info)
        .unwrap()
        .into_result()
        .unwrap();
}

#[test]
fn test_validate_ethereum() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let transaction_validator = binding_test
        .as_module_bundle::<rooch_types::framework::transaction_validator::TransactionValidator>(
    );
    let address_mapping =
        binding_test.as_module_bundle::<rooch_types::framework::address_mapping::AddressMapping>();

    let keystore = InMemKeystore::<EthereumAddress>::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = U256::zero();
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let action_bytes =
        Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    let tx_data = EthereumTransactionData::new_for_test(sender, sequence_number, action_bytes);
    let (_, _sig) = keystore
        .sign_transaction(
            &sender,
            tx_data.clone(),
            KeyPairType::EthereumKeyPairType,
            Some("".to_owned()),
        )
        .unwrap();
    let auth_info = tx_data.authenticator_info().unwrap();
    let multichain_address = MultiChainAddress::from(sender);
    let resolved_sender = address_mapping
        .resovle_or_generate(multichain_address)
        .expect("Resolve multichain address should succeed");
    let move_tx = tx_data
        .construct_moveos_transaction(resolved_sender)
        .unwrap();

    transaction_validator
        .validate(&move_tx.ctx, auth_info)
        .unwrap()
        .into_result()
        .unwrap();
}

#[test]
fn test_session_key_rooch() {
    // tracing_subscriber::fmt::init();
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();

    let mut keystore = InMemKeystore::<RoochAddress>::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = 0;

    let session_auth_key = keystore.generate_session_key(&sender, None).unwrap();

    let session_scope = SessionScope::new(
        ROOCH_FRAMEWORK_ADDRESS,
        Empty::MODULE_NAME.as_str(),
        Empty::EMPTY_FUNCTION_NAME.as_str(),
    );
    let max_inactive_interval = 100;
    let action = rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
        session_auth_key.as_ref().to_vec(),
        session_scope.clone(),
        max_inactive_interval,
    );
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);
    let tx = keystore
        .sign_transaction(
            &sender,
            tx_data,
            KeyPairType::RoochKeyPairType,
            Some("".to_owned()),
        )
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
    assert_eq!(session_key.max_inactive_interval, max_inactive_interval);

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
            ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, ident_str!("account").to_owned()),
            ident_str!("create_account_entry").to_owned(),
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
            let (_category, reason) = moveos_types::move_std::error::explain(*code);
            // ErrorFunctionCallBeyondSessionScope = 5
            assert_eq!(reason, 5, "expect ErrorFunctionCallBeyondSessionScope");
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
    let tx = keystore
        .sign_transaction(
            &sender,
            tx_data,
            KeyPairType::RoochKeyPairType,
            Some("".to_owned()),
        )
        .unwrap();
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
            let (_category, reason) = moveos_types::move_std::error::explain(*code);
            // ErrorSessionIsExpired = 4
            assert_eq!(reason, 4, "expect ErrorSessionIsExpired");
        }
        _ => {
            panic!("Expect move abort")
        }
    }
}
