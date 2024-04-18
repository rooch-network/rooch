// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::{MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("genesis");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisContext {
    pub chain_id: u64,
    /// The timestamp of the genesis, in microseconds
    pub timestamp: u64,
    /// Sequencer account
    pub sequencer: AccountAddress,
}

impl MoveStructType for GenesisContext {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GenesisContext");
}

impl MoveStructState for GenesisContext {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Address,
        ])
    }
}

impl GenesisContext {
    pub fn new(chain_id: u64, timestamp: u64, sequencer: AccountAddress) -> Self {
        Self {
            chain_id,
            timestamp,
            sequencer,
        }
    }
}
