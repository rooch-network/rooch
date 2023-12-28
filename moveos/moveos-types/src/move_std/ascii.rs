// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVE_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use anyhow::ensure;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::{IdentStr, Identifier},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone, PartialOrd, Ord, Hash, JsonSchema)]
pub struct MoveAsciiString {
    bytes: Vec<u8>,
}

impl MoveAsciiString {
    pub fn new(bytes: Vec<u8>) -> anyhow::Result<Self> {
        ensure!(bytes.is_ascii(), "string is not ascii");
        Ok(MoveAsciiString { bytes })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<MoveAsciiString> for String {
    fn from(value: MoveAsciiString) -> Self {
        String::from_utf8_lossy(value.as_bytes()).to_string()
    }
}

impl std::fmt::Display for MoveAsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //DO not check ascii when display
        write!(
            f,
            "{}",
            std::str::from_utf8(&self.bytes).map_err(|_| std::fmt::Error)?
        )
    }
}

impl FromStr for MoveAsciiString {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(s.is_ascii(), "string is not ascii");
        Ok(MoveAsciiString {
            bytes: s.as_bytes().to_vec(),
        })
    }
}

impl MoveStructType for MoveAsciiString {
    const ADDRESS: AccountAddress = MOVE_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("ascii");
    const STRUCT_NAME: &'static IdentStr = ident_str!("String");
}

impl MoveStructState for MoveAsciiString {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8))])
    }
}

impl Serialize for MoveAsciiString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.to_string().as_str())
        } else {
            serializer.serialize_bytes(&self.bytes)
        }
    }
}

impl<'de> Deserialize<'de> for MoveAsciiString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Ok(MoveAsciiString::from_str(s.as_str()).map_err(serde::de::Error::custom)?)
        } else {
            //TODO should we check ascii here?
            let bytes = Vec::<u8>::deserialize(deserializer)?;
            Ok(MoveAsciiString { bytes })
        }
    }
}

impl TryFrom<AnnotatedMoveStruct> for MoveAsciiString {
    type Error = anyhow::Error;

    fn try_from(value: AnnotatedMoveStruct) -> Result<Self, Self::Error> {
        let mut annotated_move_struct = value;
        let (field_name, field_value) = annotated_move_struct
            .value
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Invalid MoveAsciiString"))?;
        debug_assert!(field_name.as_str() == "bytes");
        let bytes = match field_value {
            AnnotatedMoveValue::Bytes(bytes) => bytes,
            _ => return Err(anyhow::anyhow!("Invalid MoveAsciiString")),
        };
        Ok(MoveAsciiString { bytes })
    }
}

impl TryFrom<MoveAsciiString> for Identifier {
    type Error = anyhow::Error;

    fn try_from(value: MoveAsciiString) -> Result<Self, Self::Error> {
        Identifier::new(value.to_string())
    }
}
