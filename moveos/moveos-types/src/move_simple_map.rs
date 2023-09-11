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

/// `SimpleMap` is represented `moveos_std::simple_map::SimpleMap` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SimpleMap<Key, Value> {
    pub data: Vec<Element<Key, Value>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Element<Key, Value> {
    pub key: Key,
    pub value: Value,
}

impl<Key, Value> SimpleMap<Key, Value> {
    pub fn create() -> Self {
        Self { data: vec![] }
    }
}

impl<Key, Value> SimpleMap<Key, Value>
where
    Key: MoveState + std::cmp::PartialEq<Key>,
    Value: MoveState,
{
    pub fn add(&mut self, key: Key, value: Value) {
        if self.contains(&key) {
            panic!("Key already exists");
        }
        self.data.push(Element::<Key, Value> { key, value });
    }

    pub fn borrow(&self, key: &Key) -> Option<&Value> {
        for element in &self.data {
            if &element.key == key {
                return Some(&element.value);
            }
        }
        None
    }

    pub fn contains(&self, key: &Key) -> bool {
        for element in &self.data {
            if &element.key == key {
                return true;
            }
        }
        false
    }

    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}

impl<Key, Value> MoveStructType for Element<Key, Value>
where
    Key: MoveState,
    Value: MoveState,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("simple_map");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Element");

    fn type_params() -> Vec<TypeTag> {
        vec![Key::type_tag(), Value::type_tag()]
    }
}

impl<Key, Value> MoveStructState for Element<Key, Value>
where
    Key: MoveState,
    Value: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![Key::type_layout(), Value::type_layout()])
    }
}

impl<Key, Value> MoveStructType for SimpleMap<Key, Value>
where
    Key: MoveState,
    Value: MoveState,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;

    const MODULE_NAME: &'static IdentStr = ident_str!("simple_map");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SimpleMap");

    fn type_params() -> Vec<TypeTag> {
        vec![Key::type_tag(), Value::type_tag()]
    }
}

impl<Key, Value> MoveStructState for SimpleMap<Key, Value>
where
    Key: MoveState,
    Value: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(
            MoveTypeLayout::Struct(Element::<Key, Value>::struct_layout()),
        ))])
    }
}
