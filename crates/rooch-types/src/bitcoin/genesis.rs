// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::{MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("genesis");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinGenesisContext {
    /// The bitcoin network
    pub network: u8,
}

impl MoveStructType for BitcoinGenesisContext {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinGenesisContext");
}

impl MoveStructState for BitcoinGenesisContext {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U8,
        ])
    }
}

impl BitcoinGenesisContext {
    pub fn new(network: u8) -> Self {
        Self { network }
    }
}
