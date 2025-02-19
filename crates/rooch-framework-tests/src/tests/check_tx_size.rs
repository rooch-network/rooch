// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test::RustBindingTest;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveValue;
use moveos_types::addresses::MOVEOS_STD_ADDRESS;
use moveos_types::move_types::FunctionId;
use moveos_types::transaction::MoveAction;
use rand::Rng;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_types::transaction::RoochTransactionData;

#[tokio::test]
async fn check_tx_size() {
    let function_id = FunctionId::new(
        ModuleId::new(
            MOVEOS_STD_ADDRESS,
            Identifier::new("module_store".to_owned()).unwrap(),
        ),
        Identifier::new("publish_package_entry".to_owned()).unwrap(),
    );

    let big_size_payload = generate_random_bytes(1024 * 1024 * 10);
    let arg_bytes = vec![MoveValue::vector_u8(big_size_payload)
        .simple_serialize()
        .unwrap()];

    let action = MoveAction::new_function_call(function_id, vec![], arg_bytes);

    let keystore = InMemKeystore::new_insecure_for_tests(1);
    let sender = keystore.addresses()[0];
    let tx_data = RoochTransactionData::new_for_test(sender, 0, action);
    let tx = keystore.sign_transaction(&sender, tx_data, None).unwrap();

    let mut binding_test = RustBindingTest::new().unwrap();
    let error = binding_test.execute_as_result(tx).unwrap_err();
    assert_eq!(
        format!("{}", error),
        "VMError with status OUT_OF_GAS at location UNDEFINED"
    );
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; size];

    rng.fill(&mut bytes[..]);

    bytes
}
