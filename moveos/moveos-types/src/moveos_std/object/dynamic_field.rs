// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveState, MoveStructState, MoveStructType},
};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    value::MoveStructLayout,
};
use move_vm_types::values::{Struct, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub const DYNAMIC_FIELD_STRUCT_NAME: &IdentStr = ident_str!("DynamicField");

/// A wrapper of Object dynamic field value, mirroring `DynamicField<N, V>` in `object.move`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicField<N, V> {
    pub name: N,
    pub value: V,
}

impl<N, V> DynamicField<N, V> {
    pub fn new(name: N, value: V) -> Self {
        Self { name, value }
    }
}

impl<N, V> MoveStructType for DynamicField<N, V>
where
    N: MoveState,
    V: MoveState,
{
    const ADDRESS: move_core_types::account_address::AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = super::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = DYNAMIC_FIELD_STRUCT_NAME;

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![N::type_tag(), V::type_tag()],
        }
    }
}

impl<N, V> MoveStructState for DynamicField<N, V>
where
    N: MoveState,
    V: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![N::type_layout(), V::type_layout()])
    }

    fn from_runtime_value_struct(value: Struct) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?.collect::<Vec<Value>>();
        debug_assert!(fields.len() == 2, "Fields of Field struct must be 2");
        let v = fields.pop().unwrap();
        let k = fields.pop().unwrap();
        Ok(DynamicField {
            name: N::from_runtime_value(k)?,
            value: V::from_runtime_value(v)?,
        })
    }
}

pub fn is_dynamic_field_type(tag: &TypeTag) -> bool {
    match tag {
        TypeTag::Struct(tag) => is_field_struct_tag(tag),
        _ => false,
    }
}

pub fn is_field_struct_tag(tag: &StructTag) -> bool {
    tag.address == MOVEOS_STD_ADDRESS
        && tag.module.as_ref() == super::MODULE_NAME
        && tag.name.as_ref() == DYNAMIC_FIELD_STRUCT_NAME
}

pub fn construct_dynamic_field_struct_tag(name_tag: TypeTag, value_tag: TypeTag) -> StructTag {
    StructTag {
        address: MOVEOS_STD_ADDRESS,
        module: super::MODULE_NAME.to_owned(),
        name: DYNAMIC_FIELD_STRUCT_NAME.to_owned(),
        type_params: vec![name_tag, value_tag],
    }
}
