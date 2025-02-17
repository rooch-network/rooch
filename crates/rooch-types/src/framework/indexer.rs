// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::value::MoveTypeLayout;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object;
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::string::MoveString,
    moveos_std::object::ObjectID,
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("indexer");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldIndexerData {
    pub path: MoveString,
    pub ext: MoveString,
}

impl MoveStructType for FieldIndexerData {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("FieldIndexerData");
}

impl MoveStructState for FieldIndexerData {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            MoveString::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldIndexerTablePlaceholder {
    _placeholder: bool,
}

impl MoveStructType for FieldIndexerTablePlaceholder {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("FieldIndexerTablePlaceholder");
}

impl MoveStructState for FieldIndexerTablePlaceholder {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}

/// Rust bindings for RoochFramework indexer module
pub struct IndexerModule<'a> {
    _caller: &'a dyn MoveFunctionCaller,
}

impl IndexerModule<'_> {
    pub fn field_indexer_object_id() -> ObjectID {
        object::named_object_id(&FieldIndexerTablePlaceholder::struct_tag())
    }
}

impl<'a> ModuleBinding<'a> for IndexerModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { _caller: caller }
    }
}
