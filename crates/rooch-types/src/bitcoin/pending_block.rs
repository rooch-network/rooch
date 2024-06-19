// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::BITCOIN_MOVE_ADDRESS;
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::tx_context::TxContext,
    state::MoveStructState,
    state::MoveStructType,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("pending_block");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTxs {
    pub block_hash: AccountAddress,
    pub txs: Vec<AccountAddress>,
}

impl MoveStructType for PendingTxs {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("PendingTxs");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for PendingTxs {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address,
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::Address,
            )),
        ])
    }
}

/// Rust bindings for BitcoinMove bitcoin module
pub struct PendingBlockModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> PendingBlockModule<'a> {
    pub const GET_READY_PENDING_TXS_FUNCTION_NAME: &'static IdentStr =
        ident_str!("get_ready_pending_txs");
    pub const GET_LATEST_BLOCK_HEIGHT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("get_latest_block_height");

    pub fn get_ready_pending_txs(&self) -> Result<Option<PendingTxs>> {
        let call =
            Self::create_function_call(Self::GET_READY_PENDING_TXS_FUNCTION_NAME, vec![], vec![]);
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let pending_txs_opt =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<MoveOption<PendingTxs>>(&value.value)
                        .expect("should be a valid MoveOption<PendingTxs>")
                })?;
        Ok(pending_txs_opt.into())
    }

    pub fn get_latest_block_height(&self) -> Result<Option<u64>> {
        let call =
            Self::create_function_call(Self::GET_LATEST_BLOCK_HEIGHT_FUNCTION_NAME, vec![], vec![]);
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let height = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<MoveOption<u64>>(&value.value)
                    .expect("should be a valid MoveOption<u64>")
            })?;
        Ok(height.into())
    }
}

impl<'a> ModuleBinding<'a> for PendingBlockModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
