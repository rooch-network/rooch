// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result,ensure, bail};
use move_core_types::{move_resource::MoveStructType, value::{MoveStructLayout}, language_storage::{StructTag, TypeTag}};
use serde::{de::DeserializeOwned, Serialize, Deserialize};

use crate::{object::{Object, RawObject, self}, addresses::MOVEOS_STD_ADDRESS};


/// `Box` is represent the `moveos_std::raw_table::Box<V>`
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Box{
    pub val: Vec<u8>,
}

/// `State` is represent state in MoveOS statedb, it can be a Move module or a Move resource
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct State {
    pub box_value: Box,
    /// the type of state, always be a StructTag of `moveos_std::raw_table::Box<V>`
    pub box_type: StructTag,
}

/// Move State is a trait that is used to represent the state of a Move Resource in Rust
pub trait MoveState : MoveStructType + DeserializeOwned + Serialize {

    fn move_layout() -> MoveStructLayout;

}

impl State {

    pub fn new(val: Vec<u8>, box_type: StructTag) -> Self {
        Self {
            box_value: Box{val},
            box_type,
        }
    }

    /// The box type's first type param is the value type
    pub fn val_type(&self) -> &TypeTag {
        &self.box_type.type_params[0]
    }
    
    pub fn is_object(&self) -> bool {
        let val_type = self.val_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                struct_tag.address == *MOVEOS_STD_ADDRESS &&
                struct_tag.module.as_ident_str() == object::OBJECT_MODULE_NAME &&
                struct_tag.name.as_ident_str() == object::OBJECT_STRUCT_NAME
            },
            _ => false,
        }
    }

    pub fn as_object<T>(&self) -> Result<Object<T>> where T:MoveState {
        let val_type = self.val_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                let expect_type = Object::<T>::struct_tag();
                //TODO define error code and rasie it to Move
                ensure!(struct_tag.as_ref() == &expect_type, "Expect type:{} but the state type:{}", expect_type, struct_tag);
                bcs::from_bytes(&self.box_value.val).map_err(Into::into)
            },
            _ => bail!("Expect type Object but the state type:{}", val_type),
        }
    }

    pub fn as_raw_object(&self) -> Result<RawObject> {
        ensure!(self.is_object(), "Expect type Object but the state type:{}", self.box_type);
        RawObject::from_bytes(&self.box_value.val)
    }
}

impl <T> From<T> for State where T: MoveState {
    fn from(val: T) -> Self {
        let val_type = T::struct_tag();
        let val = bcs::to_bytes(&val).expect("Serialize MoveState to bcs should success");
        Self::new(val, val_type)
    }
}
