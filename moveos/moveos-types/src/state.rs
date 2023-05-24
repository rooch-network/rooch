// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    object::{self, Object, RawObject},
};
use anyhow::{bail, ensure, Result};
use move_core_types::{
    language_storage::{StructTag, TypeTag},
    move_resource::MoveStructType,
    value::MoveStructLayout,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// `State` is represent state in MoveOS statedb, it can be a Move module or a Move Object or a Move resource or a Table value
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct State {
    /// the bytes of state
    pub value: Vec<u8>,
    /// the type of state
    pub value_type: TypeTag,
}

/// Move State is a trait that is used to represent the state of a Move Resource in Rust
/// It is like the `MoveResource` in move_core_types
pub trait MoveState: MoveStructType + DeserializeOwned + Serialize {
    fn move_layout() -> MoveStructLayout;
}

impl State {
    pub fn new(value: Vec<u8>, value_type: TypeTag) -> Self {
        Self { value, value_type }
    }

    pub fn value_type(&self) -> &TypeTag {
        &self.value_type
    }

    pub fn match_type(&self, type_tag: &TypeTag) -> bool {
        &self.value_type == type_tag
    }

    pub fn match_struct_type(&self, type_tag: &StructTag) -> bool {
        match &self.value_type {
            TypeTag::Struct(struct_tag) => struct_tag.as_ref() == type_tag,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        let val_type = self.value_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                struct_tag.address == *MOVEOS_STD_ADDRESS
                    && struct_tag.module.as_ident_str() == object::OBJECT_MODULE_NAME
                    && struct_tag.name.as_ident_str() == object::OBJECT_STRUCT_NAME
            }
            _ => false,
        }
    }

    pub fn as_object<T>(&self) -> Result<Object<T>>
    where
        T: MoveState,
    {
        self.as_move_state::<Object<T>>()
    }

    pub fn as_raw_object(&self) -> Result<RawObject> {
        ensure!(
            self.is_object(),
            "Expect type Object but the state type:{}",
            self.value_type
        );
        RawObject::from_bytes(&self.value)
    }

    pub fn as_move_state<T>(&self) -> Result<T>
    where
        T: MoveState,
    {
        let val_type = self.value_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                let expect_type = T::struct_tag();
                //TODO define error code and rasie it to Move
                ensure!(
                    struct_tag.as_ref() == &expect_type,
                    "Expect type:{} but the state type:{}",
                    expect_type,
                    struct_tag
                );
                bcs::from_bytes(&self.value).map_err(Into::into)
            }
            _ => bail!("Expect type Object but the state type:{}", val_type),
        }
    }
}

impl<T> From<T> for State
where
    T: MoveState,
{
    fn from(val: T) -> Self {
        let val_type = T::struct_tag();
        let val = bcs::to_bytes(&val).expect("Serialize MoveState to bcs should success");
        Self::new(val, TypeTag::Struct(Box::new(val_type)))
    }
}
