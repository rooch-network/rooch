// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{addresses::BITCOIN_MOVE_ADDRESS, into_address::IntoAddress};
use anyhow::Result;
use bitcoin::Txid;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("bbn");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBNStakeSeal {
    /// The stake transaction block height
    pub block_height: u64,
    /// The stake transaction hash
    pub txid: AccountAddress,
    /// The stake utxo output index
    pub vout: u32,
    pub tag: Vec<u8>,
    pub version: u64,
    pub staker_pub_key: Vec<u8>,
    pub finality_provider_pub_key: Vec<u8>,
    /// The stake time in block count
    pub staking_time: u16,
    /// The stake amount in satoshi
    pub staking_amount: u64,
}

impl MoveStructType for BBNStakeSeal {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BBNStakeSeal");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BBNStakeSeal {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Address,
            MoveTypeLayout::U32,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U16,
            MoveTypeLayout::U64,
        ])
    }
}

/// Rust bindings for BitcoinMove bitcoin module
pub struct BBNModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BBNModule<'a> {
    pub const IS_BBN_TX_FUNCTION_NAME: &'static IdentStr = ident_str!("is_bbn_tx");
    pub const PROCESS_BBN_TX_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("process_bbn_tx_entry");

    pub fn is_bbn_tx(&self, txid: Txid) -> Result<bool> {
        let call = Self::create_function_call(
            Self::IS_BBN_TX_FUNCTION_NAME,
            vec![],
            vec![txid.into_address().to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let is_bbn_tx = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value).expect("should be a valid bool")
            })
            .map_err(|e| anyhow::anyhow!("Failed to get is bbn tx: {:?}", e))?;
        Ok(is_bbn_tx.into())
    }

    pub fn create_process_bbn_tx_entry_call(&self, txid: Txid) -> Result<FunctionCall> {
        Ok(Self::create_function_call(
            Self::PROCESS_BBN_TX_ENTRY_FUNCTION_NAME,
            vec![],
            vec![txid.into_address().to_move_value()],
        ))
    }
}

impl<'a> ModuleBinding<'a> for BBNModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
