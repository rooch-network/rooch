// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{Bytes, U256};
use moveos_types::transaction::MoveAction;
use rooch_key::keystore::{AccountKeystore, InMemKeystore};
use rooch_types::address::MultiChainAddress;
use rooch_types::framework::empty::Empty;

use crate::binding_test;

// TODO: resolve conversion from rooch address to ethereum address and rooch tx to ethereum tx
#[test]
fn test_validate() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let _ethereum_validator = binding_test
        .as_module_bundle::<rooch_types::framework::ethereum_validator::EthereumValidatorModule>(
    );
    let address_mapping =
        binding_test.as_module_bundle::<rooch_types::framework::address_mapping::AddressMapping>();

    let keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let _sequence_number = U256::zero();
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let _action_bytes =
        Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    // let tx_data = EthereumTransactionData::new_for_test(sender_ethereum, sequence_number, action_bytes);
    // keystore
    //     .sign_transaction(&sender, tx_data.clone(), None)
    //     .unwrap();
    // let auth_info = tx_data.authenticator_info().unwrap();
    let multichain_address = MultiChainAddress::from(sender);
    let _resolved_sender = address_mapping
        .resovle_or_generate(multichain_address)
        .expect("Resolve multichain address should succeed");
    // let move_tx = tx_data
    //     .construct_moveos_transaction(resolved_sender)
    //     .unwrap();

    // ethereum_validator
    //     .validate(&move_tx.ctx, auth_info.authenticator.payload)
    //     .unwrap()
}
