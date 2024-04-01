// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos_std::object;
use crate::moveos_std::object::ObjectID;
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use anyhow::{anyhow, Result};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_vm_types::values::{Struct, Value};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("move_module");

/// `MoveModule` is represented `moveos_std::move_module::MoveModule` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MoveModule {
    pub byte_codes: Vec<u8>,
}

impl MoveModule {
    pub fn new(byte_codes: Vec<u8>) -> Self {
        Self { byte_codes }
    }
}

impl MoveStructType for MoveModule {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MoveModule");
}

impl MoveStructState for MoveModule {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8))])
    }

    fn from_runtime_value_struct(value: Struct) -> Result<Self>
    where
        Self: Sized,
    {
        let mut module_fields = value.unpack()?.collect::<Vec<Value>>();
        debug_assert!(
            module_fields.len() == 1,
            "Fields of Module struct must be 1, actual: {}",
            module_fields.len()
        );
        let module = module_fields.pop().unwrap();

        let byte_codes = module.value_as::<Vec<u8>>()?;
        Ok(Self { byte_codes })
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModuleStore {
    // // Move VM will auto add a bool field to the empty struct
    // // So we manually add a bool field to the struct
    _placeholder: bool,
}

impl ModuleStore {
    pub fn module_store_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for ModuleStore {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ModuleStore");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for ModuleStore {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}

/// Represents the module id
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveModuleId {
    address: AccountAddress,
    name: Identifier,
}

impl MoveModuleId {
    pub fn parse(str: &str) -> Result<Self> {
        let parts: Vec<_> = str.split("::").collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid module id"));
        }
        let address = AccountAddress::from_str(parts[0])?;
        let name = Identifier::new(parts[1])?;
        Ok(Self { address, name })
    }

    pub fn into_module_id(self) -> ModuleId {
        ModuleId::new(self.address, self.name)
    }
}

impl FromStr for MoveModuleId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
