// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::state::{MoveStructState, MoveStructType};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("genesis");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveosGenesisContext {
    /// The timestamp of the genesis, in microseconds
    pub timestamp: u64,
}

impl MoveStructType for MoveosGenesisContext {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MoveosGenesisContext");
}

impl MoveStructState for MoveosGenesisContext {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}

impl MoveosGenesisContext {
    pub fn new(timestamp: u64) -> Self {
        Self { timestamp }
    }
}
