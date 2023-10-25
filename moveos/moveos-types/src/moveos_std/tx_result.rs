// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, vm_status::KeptVMStatus,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("tx_result");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResult {
    pub executed: bool,
    pub gas_used: u64,
    pub gas_payment_account: AccountAddress,
}

impl MoveStructType for TxResult {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TxResult");
}

impl MoveStructState for TxResult {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Bool,
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Address,
        ])
    }
}

impl TxResult {
    pub fn new(status: &KeptVMStatus, gas_used: u64, gas_payment_account: AccountAddress) -> Self {
        Self {
            executed: matches!(status, KeptVMStatus::Executed),
            gas_used,
            gas_payment_account,
        }
    }
}
