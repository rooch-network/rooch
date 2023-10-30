// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    moveos_std::object::ObjectID,
    moveos_std::tx_context::TxContext,
    state::{MoveStructState, MoveStructType},
    state_resolver,
};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("context");
pub const GLOBAL_OBJECT_STORAGE_HANDLE: ObjectID = state_resolver::GLOBAL_OBJECT_STORAGE_HANDLE;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StorageContext {
    pub handle: ObjectID,
}

impl MoveStructType for StorageContext {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("StorageContext");
}

impl MoveStructState for StorageContext {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Struct(ObjectID::struct_layout())])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Context {
    pub tx_context: TxContext,
    pub storage_context: StorageContext,
}

impl Context {
    /// New global storage context
    pub fn new(tx_context: TxContext) -> Self {
        Self {
            tx_context,
            storage_context: StorageContext {
                handle: GLOBAL_OBJECT_STORAGE_HANDLE,
            },
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(&self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}

impl MoveStructType for Context {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Context");
}

impl MoveStructState for Context {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Struct(TxContext::struct_layout()),
            MoveTypeLayout::Struct(StorageContext::struct_layout()),
        ])
    }
}
