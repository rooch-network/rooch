// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{address::BitcoinAddress, addresses::ROOCH_FRAMEWORK_ADDRESS};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("genesis");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisContext {
    pub chain_id: u64,
    /// Sequencer account
    pub sequencer: BitcoinAddress,
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
            BitcoinAddress::type_layout(),
        ])
    }
}

impl GenesisContext {
    pub fn new(chain_id: u64, sequencer: BitcoinAddress) -> Self {
        Self {
            chain_id,
            sequencer,
        }
    }
}
