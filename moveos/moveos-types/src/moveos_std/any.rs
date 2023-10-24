// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    move_std::string::MoveString,
    state::{MoveState, MoveStructState, MoveStructType},
};
use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// `Any` is represented `moveos_std::any::Any` in Move.
#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Any {
    pub type_name: MoveString,
    pub data: Vec<u8>,
}

impl std::fmt::Debug for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Any {{ type_name: {}, data: {} }}",
            self.type_name,
            hex::encode(&self.data)
        )
    }
}

impl MoveStructType for Any {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("any");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Any");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveStructState for Any {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Struct(MoveString::struct_layout()),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }
}

pub trait AnyTrait {
    fn new(type_name: MoveString, data: Vec<u8>) -> Self
    where
        Self: Sized;

    fn pack<T>(value: T) -> Result<Self>
    where
        T: MoveState,
        Self: Sized,
    {
        Ok(Self::new(
            MoveString::from_str(T::type_tag().to_canonical_string().as_str())?,
            value.to_bytes(),
        ))
    }

    fn into_inner(self) -> (MoveString, Vec<u8>);

    fn unpack<T>(self) -> Result<T>
    where
        T: MoveState,
        Self: Sized,
    {
        let (type_name, data) = self.into_inner();
        if type_name
            != MoveString::from_str(T::type_tag().to_canonical_string().as_str())
                .expect("Failed to convert type tag to string")
        {
            bail!(
                "Invalid type, expected {}, got {}",
                type_name,
                T::type_tag().to_canonical_string()
            );
        }
        T::from_bytes(&data)
    }
}

impl AnyTrait for Any {
    fn new(type_name: MoveString, data: Vec<u8>) -> Self
    where
        Self: Sized,
    {
        Self { type_name, data }
    }

    fn into_inner(self) -> (MoveString, Vec<u8>) {
        (self.type_name, self.data)
    }
}
