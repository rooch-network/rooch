// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_types::chain_id::RoochChainID;

#[test]
fn test_chain_id() {
    let _ = tracing_subscriber::fmt::try_init();
    let binding_test = binding_test::RustBindingTest::new().unwrap();
    let chain_id_module =
        binding_test.as_module_binding::<rooch_types::framework::chain_id::ChainIDModule>();
    let chain_id = chain_id_module.chain_id().unwrap();
    assert_eq!(chain_id, RoochChainID::LOCAL.chain_id().id());
}
