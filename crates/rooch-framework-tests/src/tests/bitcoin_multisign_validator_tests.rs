// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use move_core_types::u256::U256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::state::MoveStructType;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use rooch_types::bitcoin::bitcoin_multisign_validator::BitcoinMultisignValidatorModule;
use rooch_types::bitcoin::multisign_account::{self, MultisignAccountModule};
use rooch_types::crypto::{RoochKeyPair, RoochSignature};
use rooch_types::framework::auth_payload::{MultisignAuthPayload, SignData};
use rooch_types::framework::auth_validator::BuiltinAuthValidator;
use rooch_types::framework::empty::Empty;
use rooch_types::framework::gas_coin::GasCoin;
use rooch_types::framework::transfer::TransferModule;
use rooch_types::transaction::rooch::RoochTransactionData;
use rooch_types::transaction::{Authenticator, RoochTransaction};

#[tokio::test]
async fn test_validate() {
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    let root = binding_test.root().clone();

    let kp1 = RoochKeyPair::generate_secp256k1();
    let kp2 = RoochKeyPair::generate_secp256k1();
    let kp3 = RoochKeyPair::generate_secp256k1();

    let u1 = kp1.public().bitcoin_address().unwrap().to_rooch_address();
    //let u2 = kp2.public().bitcoin_address().unwrap().to_rooch_address();
    //let u3 = kp3.public().bitcoin_address().unwrap().to_rooch_address();

    let pubkeys = vec![
        kp1.bitcoin_public_key().unwrap(),
        kp2.bitcoin_public_key().unwrap(),
        kp3.bitcoin_public_key().unwrap(),
    ];

    let pubkeys = pubkeys
        .into_iter()
        .map(|pk| pk.to_bytes())
        .collect::<Vec<_>>();

    let bitcoin_address_from_rust =
        multisign_account::generate_multisign_address(2, pubkeys.clone()).unwrap();
    //println!("bitcoin_address_from_rust: {}", bitcoin_address_from_rust);

    //Initialize the multisign account
    let action = MultisignAccountModule::initialize_multisig_account_action(2, pubkeys.to_vec());
    let tx_data = RoochTransactionData::new_for_test(u1, 0, action);
    let tx = tx_data.sign(&kp1);
    binding_test.execute(tx).unwrap();

    let multisign_address = bitcoin_address_from_rust.to_rooch_address();

    //transfer gas free to multisign account

    let gas_action = TransferModule::create_transfer_coin_action(
        GasCoin::struct_tag(),
        multisign_address.into(),
        U256::from(1000000000u128),
    );

    let gas_tx_data = RoochTransactionData::new_for_test(u1, 1, gas_action);
    let gas_tx = gas_tx_data.sign(&kp1);
    binding_test.execute(gas_tx).unwrap();

    let sender = multisign_address;
    let sequence_number = 0;
    let action = MoveAction::new_function_call(Empty::empty_function_id(), vec![], vec![]);
    let tx_data = RoochTransactionData::new_for_test(sender, sequence_number, action);

    let sign_data = SignData::new_with_default(&tx_data);
    let data_hash = sign_data.data_hash();

    let signature1 = kp1.sign(data_hash.as_bytes());
    let signature2 = kp2.sign(data_hash.as_bytes());

    let message_info = sign_data.message_info_without_tx_hash();
    let payload = MultisignAuthPayload {
        signatures: vec![
            signature1.signature_bytes().to_vec(),
            signature2.signature_bytes().to_vec(),
        ],
        message_prefix: sign_data.message_prefix,
        message_info,
        public_keys: pubkeys[0..2].to_vec(),
    };

    let authenticator = Authenticator::new(
        BuiltinAuthValidator::BitcoinMultisign.flag().into(),
        bcs::to_bytes(&payload).unwrap(),
    );

    let tx = RoochTransaction::new(tx_data, authenticator);

    let auth_info = tx.authenticator_info();

    //Test the validate function
    {
        let move_tx: MoveOSTransaction = tx.clone().into_moveos_transaction(root);

        let validator_caller = binding_test.as_module_binding::<BitcoinMultisignValidatorModule>();
        let result = validator_caller.validate(&move_tx.ctx, auth_info.authenticator.payload);
        assert!(result.is_ok());
    }

    //Execute the multisign transaction
    binding_test.execute(tx).unwrap();
}
