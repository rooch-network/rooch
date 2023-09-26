// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{ident_str, identifier::IdentStr};
use moveos_types::state::MoveStructType;

pub const MODULE_NAME: &IdentStr = ident_str!("gas_coin");

#[derive(Debug, Clone)]
pub struct GasCoin;

impl MoveStructType for GasCoin {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasCoin");
}
