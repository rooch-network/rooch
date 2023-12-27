// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveState, MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

/// `SimpleMultiMap` is represented `moveos_std::simple_multimap::SimpleMultiMap` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SimpleMultiMap<Key, Value> {
    pub data: Vec<Element<Key, Value>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Element<Key, Value> {
    pub key: Key,
    pub value: Vec<Value>,
}

impl<Key, Value> SimpleMultiMap<Key, Value> {
    pub fn create() -> Self {
        Self { data: vec![] }
    }
}

impl<Key, Value> SimpleMultiMap<Key, Value>
where
    Key: MoveState + std::cmp::PartialEq<Key>,
    Value: MoveState,
{
}

impl<Key, Value> MoveStructType for Element<Key, Value>
where
    Key: MoveState,
    Value: MoveState,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("simple_multimap");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Element");

    fn type_params() -> Vec<TypeTag> {
        vec![Key::type_tag(), Value::type_tag()]
    }
}

impl<Key, Value> MoveStructState for Element<Key, Value>
where
    Key: MoveState + Serialize,
    Value: MoveState + Serialize,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            Key::type_layout(),
            MoveTypeLayout::Vector(Box::new(Value::type_layout())),
        ])
    }
}

impl<Key, Value> MoveStructType for SimpleMultiMap<Key, Value>
where
    Key: MoveState,
    Value: MoveState,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("simple_multimap");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SimpleMultiMap");

    fn type_params() -> Vec<TypeTag> {
        vec![Key::type_tag(), Value::type_tag()]
    }
}

impl<Key, Value> MoveStructState for SimpleMultiMap<Key, Value>
where
    Key: MoveState + Serialize,
    Value: MoveState + Serialize,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(
            MoveTypeLayout::Struct(Element::<Key, Value>::struct_layout()),
        ))])
    }
}
