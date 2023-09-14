// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{Bytes, U256};
use fastcrypto::secp256k1::recoverable::Secp256k1RecoverableKeyPair;
use moveos_types::transaction::MoveAction;
use rooch_key::keystore::{AccountKeystore, InMemKeystore};
use rooch_types::address::{EthereumAddress, MultiChainAddress};
use rooch_types::framework::empty::Empty;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::ethereum::EthereumTransactionData;
use rooch_types::transaction::AbstractTransaction;

use crate::binding_test;

#[test]
fn test_validate() {
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let ethereum_validator = binding_test
        .as_module_bundle::<rooch_types::framework::ethereum_validator::EthereumValidatorModule>(
    );
    let address_mapping =
        binding_test.as_module_bundle::<rooch_types::framework::address_mapping::AddressMapping>();

    let keystore =
        InMemKeystore::<EthereumAddress, Secp256k1RecoverableKeyPair>::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let sequence_number = U256::zero();
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let action_bytes =
        Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    let tx_data = EthereumTransactionData::new_for_test(sender, sequence_number, action_bytes);
    keystore
        .sign_transaction(&sender, tx_data.clone(), RoochMultiChainID::ETHER)
        .unwrap();
    let auth_info = tx_data.authenticator_info().unwrap();
    let multichain_address = MultiChainAddress::from(sender);
    let resolved_sender = address_mapping
        .resovle_or_generate(multichain_address)
        .expect("Resolve multichain address should succeed");
    let move_tx = tx_data
        .construct_moveos_transaction(resolved_sender)
        .unwrap();

    ethereum_validator
        .validate(&move_tx.ctx, auth_info.authenticator.payload)
        .unwrap()
}
