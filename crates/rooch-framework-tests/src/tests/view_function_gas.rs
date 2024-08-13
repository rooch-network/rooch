// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::StatusCode;

use moveos_types::h256::H256;
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::transaction::FunctionCall;
use rooch_types::framework::empty::Empty;

use crate::binding_test;

#[tokio::test]
async fn view_function_gas() {
    let empty_call = FunctionCall::new(
        Empty::function_id(Empty::EMPTY_FUNCTION_NAME),
        vec![],
        vec![],
    );

    let zero_gas_context = TxContext::new(AccountAddress::random(), 0, 0, H256::random(), 1);

    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let result = binding_test
        .call_function(&zero_gas_context, empty_call)
        .unwrap();
    assert_eq!(result.vm_status.status_code(), StatusCode::OUT_OF_GAS)
}
