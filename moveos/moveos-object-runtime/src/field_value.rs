// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    ident_str, identifier::IdentStr, language_storage::StructTag, value::MoveStructLayout,
};
use move_vm_types::values::{Struct, Value};
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS,
    moveos_std::object,
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub(crate) const FIELD_VALUE_STRUCT_NAME: &IdentStr = ident_str!("FieldValue");

/// A wrapper of Object dynamic field value, mirroring `FieldValue<V>` in `object.move`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldValue<V> {
    pub(crate) val: V,
}

impl<V> MoveStructType for FieldValue<V>
where
    V: MoveState,
{
    const ADDRESS: move_core_types::account_address::AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = object::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = FIELD_VALUE_STRUCT_NAME;

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![V::type_tag()],
        }
    }
}

impl<V> MoveStructState for FieldValue<V>
where
    V: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![V::type_layout()])
    }

    fn from_runtime_value_struct(value: Struct) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?.collect::<Vec<Value>>();
        debug_assert!(fields.len() == 1, "Fields of FieldValue struct must be 1");
        let v = fields.pop().unwrap();
        Ok(FieldValue {
            val: V::from_runtime_value(v)?,
        })
    }
}
