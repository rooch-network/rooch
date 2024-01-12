// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// use ethers::types::{Bytes, U256};
// use moveos_types::module_binding::MoveFunctionCaller;
// use moveos_types::transaction::MoveAction;
// use rooch_key::keystore::account_keystore::AccountKeystore;
// use rooch_key::keystore::memory_keystore::InMemKeystore;
// use rooch_types::framework::empty::Empty;
// use rooch_types::transaction::ethereum::EthereumTransaction;
// use rooch_types::transaction::AbstractTransaction;
//
// use crate::binding_test;

#[test]
fn test_validate() {
    // TODO: wait cli support eth
    // let binding_test = binding_test::RustBindingTest::new().unwrap();
    // let ethereum_validator = binding_test
    //     .as_module_binding::<rooch_types::framework::ethereum_validator::EthereumValidatorModule>(
    // );
    // let address_mapping =
    //     binding_test.as_module_binding::<rooch_types::framework::address_mapping::AddressMapping>();
    //
    // let keystore = InMemKeystore::new_insecure_for_tests(1);
    // let sender = keystore.addresses()[0];
    // let sequence_number = U256::zero();
    // let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    // let action_bytes =
    //     Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    // let tx = EthereumTransaction::new_for_test(sender, sequence_number, action_bytes);
    //
    // let multi_chain_address_sender = tx.sender();
    // let resolved_sender = address_mapping
    //     .resolve_or_generate(multi_chain_address_sender.clone())
    //     .unwrap();
    // let authenticator = tx.authenticator_info().unwrap();
    // let moveos_tx = tx.construct_moveos_transaction(resolved_sender).unwrap();
    //
    // ethereum_validator
    //     .validate(&moveos_tx.ctx, authenticator.authenticator.payload)
    //     .unwrap()
}
