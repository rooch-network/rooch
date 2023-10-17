// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    move_std::ascii::MoveAsciiString,
    move_std::option::MoveOption,
    state::{MoveStructState, MoveStructType},
    transaction::MoveAction,
};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("tx_meta");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMeta {
    pub action_type: u8,
    pub function_meta: MoveOption<FunctionCallMeta>,
}

impl MoveStructType for TxMeta {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TxMeta");
}

impl MoveStructState for TxMeta {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U8,
            MoveOption::<FunctionCallMeta>::type_layout(),
        ])
    }
}

impl TxMeta {
    pub fn new(action_type: u8, function_meta: MoveOption<FunctionCallMeta>) -> Self {
        Self {
            action_type,
            function_meta,
        }
    }

    pub fn new_from_move_action(move_action: &MoveAction) -> Self {
        let function_meta = match move_action {
            MoveAction::Function(call) => Some(FunctionCallMeta {
                module_address: *call.function_id.module_id.address(),
                module_name: MoveAsciiString::new(
                    call.function_id.module_id.name().as_bytes().to_vec(),
                )
                .expect("module name must be valid ascii"),
                function_name: MoveAsciiString::new(
                    call.function_id.function_name.as_bytes().to_vec(),
                )
                .expect("module name must be valid ascii"),
            }),
            _ => None,
        };
        Self {
            action_type: move_action.action_type(),
            function_meta: function_meta.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallMeta {
    pub module_address: AccountAddress,
    pub module_name: MoveAsciiString,
    pub function_name: MoveAsciiString,
}

impl MoveStructType for FunctionCallMeta {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("FunctionCallMeta");
}

impl MoveStructState for FunctionCallMeta {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address,
            MoveAsciiString::type_layout(),
            MoveAsciiString::type_layout(),
        ])
    }
}
