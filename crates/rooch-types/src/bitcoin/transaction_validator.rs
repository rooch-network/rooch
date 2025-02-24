// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use framework_types::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};

use crate::into_address::IntoAddress;
use moveos_types::h256::H256;
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
};

/// Rust bindings for BitcoinMove transaction_validator module
pub struct TransactionValidator<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> TransactionValidator<'a> {
    pub const VALIDATE_L1_TX_FUNCTION_NAME: &'static IdentStr = ident_str!("validate_l1_tx");

    pub fn validate_l1_tx(
        &self,
        ctx: &TxContext,
        tx_hash: H256,
        _payload: Vec<u8>,
    ) -> Result<bool> {
        let tx_validator_call = Self::create_function_call(
            Self::VALIDATE_L1_TX_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::Address(tx_hash.into_address()),
                MoveValue::vector_u8(vec![]),
            ],
        );

        let result = self
            .caller
            .call_function(ctx, tx_validator_call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value).expect("should be a valid bool")
            })
            .map_err(|e| anyhow::anyhow!("Failed to validate l1 tx: {:?}", e))?;
        Ok(result)
    }
}

impl<'a> ModuleBinding<'a> for TransactionValidator<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("transaction_validator");
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
