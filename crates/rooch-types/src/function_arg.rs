// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::error::RoochError;
use anyhow::{anyhow, Result};
use move_command_line_common::{
    address::ParsedAddress,
    parser::Parser,
    types::ParsedStructType,
    values::{ParsableValue, ValueToken},
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
    u256::U256,
    value::{MoveStruct, MoveValue},
};
use moveos_types::{
    move_types::FunctionId,
    moveos_std::object::{self, ObjectID},
};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionArgType {
    Address,
    Bool,
    ObjectID,
    Object,
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Raw,
    Vector(Box<FunctionArgType>),
}

impl Display for FunctionArgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionArgType::Address => write!(f, "address"),
            FunctionArgType::Bool => write!(f, "bool"),
            FunctionArgType::ObjectID => write!(f, "object_id"),
            FunctionArgType::Object => write!(f, "object"),
            FunctionArgType::String => write!(f, "string"),
            FunctionArgType::U8 => write!(f, "u8"),
            FunctionArgType::U16 => write!(f, "u16"),
            FunctionArgType::U32 => write!(f, "u32"),
            FunctionArgType::U64 => write!(f, "u64"),
            FunctionArgType::U128 => write!(f, "u128"),
            FunctionArgType::U256 => write!(f, "u256"),
            FunctionArgType::Raw => write!(f, "raw"),
            FunctionArgType::Vector(inner) => write!(f, "vector<{}>", inner),
        }
    }
}

impl FunctionArgType {
    fn parse_arg(&self, arg: &str) -> Result<FunctionArg> {
        match self {
            FunctionArgType::Address => ParsedAddress::parse(arg).map(FunctionArg::Address),
            FunctionArgType::Bool => Ok(FunctionArg::Bool(bool::from_str(arg)?)),
            FunctionArgType::ObjectID => Ok(FunctionArg::ObjectID(ParsedObjectID::from_str(arg)?)),
            FunctionArgType::Object => Ok(FunctionArg::Object(ParsedObjectID::from_str(arg)?)),
            FunctionArgType::String => Ok(FunctionArg::String(arg.to_string())),
            FunctionArgType::U8 => Ok(FunctionArg::U8(u8::from_str(arg)?)),
            FunctionArgType::U16 => Ok(FunctionArg::U16(u16::from_str(arg)?)),
            FunctionArgType::U32 => Ok(FunctionArg::U32(u32::from_str(arg)?)),
            FunctionArgType::U64 => Ok(FunctionArg::U64(u64::from_str(arg)?)),
            FunctionArgType::U128 => Ok(FunctionArg::U128(u128::from_str(arg)?)),
            FunctionArgType::U256 => Ok(FunctionArg::U256(U256::from_str(arg)?)),
            FunctionArgType::Raw => Ok(FunctionArg::Raw(hex::decode(arg)?)),
            FunctionArgType::Vector(inner) => {
                let mut parsed_args = vec![];
                let args = arg.split(',');
                for arg in args {
                    if !arg.is_empty() {
                        parsed_args.push(inner.parse_arg(arg)?);
                    }
                }
                Ok(FunctionArg::Vector(inner.clone(), parsed_args))
            }
        }
    }
}

impl FromStr for FunctionArgType {
    type Err = RoochError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "address" => Ok(FunctionArgType::Address),
            "bool" => Ok(FunctionArgType::Bool),
            "object_id" => Ok(FunctionArgType::ObjectID),
            "object" => Ok(FunctionArgType::Object),
            "string" => Ok(FunctionArgType::String),
            "u8" => Ok(FunctionArgType::U8),
            "u16" => Ok(FunctionArgType::U16),
            "u32" => Ok(FunctionArgType::U32),
            "u64" => Ok(FunctionArgType::U64),
            "u128" => Ok(FunctionArgType::U128),
            "u256" => Ok(FunctionArgType::U256),
            "raw" => Ok(FunctionArgType::Raw),
            str => {
                // If it's a vector, go one level inside
                if str.starts_with("vector<") && str.ends_with('>') {
                    let arg = FunctionArgType::from_str(&str[7..str.len() - 1])?;

                    if arg == FunctionArgType::Raw {
                        return Err(RoochError::CommandArgumentError(
                            "vector<raw> is not supported".to_owned(),
                        ));
                    } else if matches!(arg, FunctionArgType::Vector(_)) {
                        return Err(RoochError::CommandArgumentError(
                            "nested vector<vector<_>> is not supported".to_owned(),
                        ));
                    }

                    Ok(FunctionArgType::Vector(Box::new(arg)))
                } else {
                    Err(RoochError::CommandArgumentError(format!("Invalid arg type '{}'.  Must be one of: ['address','bool','object_id','string','u8','u16','u32','u64','u128','u256','vector<inner_type>']", str)))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParsedModuleId {
    pub address: ParsedAddress,
    pub name: Identifier,
}

impl ParsedModuleId {
    pub fn parse(str: &str) -> Result<Self> {
        let parts: Vec<_> = str.split("::").collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid module id"));
        }
        let address = ParsedAddress::parse(parts[0])?;
        let name = Identifier::new(parts[1])?;
        Ok(Self { address, name })
    }

    pub fn into_module_id(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> Result<ModuleId> {
        Ok(ModuleId::new(
            self.address.into_account_address(mapping)?,
            self.name,
        ))
    }
}

impl FromStr for ParsedModuleId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Clone, Debug)]
pub struct ParsedFunctionId {
    pub module_id: ParsedModuleId,
    pub function_name: Identifier,
}

impl ParsedFunctionId {
    pub fn parse(str: &str) -> Result<Self> {
        let parts: Vec<_> = str.split("::").collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid function id"));
        }
        let address = ParsedAddress::parse(parts[0])?;
        let module_name = Identifier::new(parts[1])?;
        let function_name = Identifier::new(parts[2])?;
        Ok(Self {
            module_id: ParsedModuleId {
                address,
                name: module_name,
            },
            function_name,
        })
    }

    pub fn into_function_id(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> Result<FunctionId> {
        let module_id = ModuleId::new(
            self.module_id.address.into_account_address(mapping)?,
            self.module_id.name,
        );
        Ok(FunctionId::new(module_id, self.function_name))
    }
}

impl FromStr for ParsedFunctionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Clone, Debug)]
pub enum ParsedObjectID {
    ObjectID(ObjectID),
    //For named object
    StructTag(ParsedStructType),
}

impl ParsedObjectID {
    pub fn into_object_id(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> Result<ObjectID> {
        Ok(match self {
            ParsedObjectID::ObjectID(object_id) => object_id,
            ParsedObjectID::StructTag(parsed_struct_type) => {
                let struct_tag = parsed_struct_type.into_struct_tag(mapping)?;
                object::named_object_id(&struct_tag)
            }
        })
    }
}

impl FromStr for ParsedObjectID {
    type Err = RoochError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            Ok(ParsedObjectID::ObjectID(ObjectID::from_str(s)?))
        } else {
            let parsed_struct_type = ParsedStructType::parse(s)?;
            Ok(ParsedObjectID::StructTag(parsed_struct_type))
        }
    }
}

/// A parseable arg with a type separated by a colon
#[derive(Clone, Debug)]
pub enum FunctionArg {
    Address(ParsedAddress),
    Bool(bool),
    ObjectID(ParsedObjectID),
    Object(ParsedObjectID),
    String(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
    Raw(Vec<u8>),
    Vector(Box<FunctionArgType>, Vec<FunctionArg>),
}

impl FunctionArg {
    pub fn arg_type(&self) -> FunctionArgType {
        match self {
            FunctionArg::Address(_) => FunctionArgType::Address,
            FunctionArg::Bool(_) => FunctionArgType::Bool,
            FunctionArg::ObjectID(_) => FunctionArgType::ObjectID,
            FunctionArg::Object(_) => FunctionArgType::Object,
            FunctionArg::String(_) => FunctionArgType::String,
            FunctionArg::U8(_) => FunctionArgType::U8,
            FunctionArg::U16(_) => FunctionArgType::U16,
            FunctionArg::U32(_) => FunctionArgType::U32,
            FunctionArg::U64(_) => FunctionArgType::U64,
            FunctionArg::U128(_) => FunctionArgType::U128,
            FunctionArg::U256(_) => FunctionArgType::U256,
            FunctionArg::Raw(_) => FunctionArgType::Raw,
            FunctionArg::Vector(element_type, _) => FunctionArgType::Vector(element_type.clone()),
        }
    }

    pub fn to_move_value(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> Result<MoveValue> {
        Ok(match self {
            FunctionArg::Address(address) => {
                let account_address = address.into_account_address(mapping)?;
                MoveValue::Address(account_address)
            }
            FunctionArg::Bool(arg) => MoveValue::Bool(arg),
            FunctionArg::ObjectID(parsed_object_id) | FunctionArg::Object(parsed_object_id) => {
                let object_id = parsed_object_id.into_object_id(mapping)?;
                MoveValue::Address(object_id.into())
            }
            FunctionArg::String(arg) => MoveValue::vector_u8(arg.as_bytes().to_vec()),
            FunctionArg::U8(arg) => MoveValue::U8(arg),
            FunctionArg::U16(arg) => MoveValue::U16(arg),
            FunctionArg::U32(arg) => MoveValue::U32(arg),
            FunctionArg::U64(arg) => MoveValue::U64(arg),
            FunctionArg::U128(arg) => MoveValue::U128(arg),
            FunctionArg::U256(arg) => MoveValue::U256(arg),
            FunctionArg::Raw(arg) => MoveValue::vector_u8(arg),
            FunctionArg::Vector(_element_type, elements) => {
                let mut move_elements = vec![];
                for element in elements {
                    move_elements.push(element.to_move_value(mapping)?);
                }
                MoveValue::Vector(move_elements)
            }
        })
    }

    pub fn into_bytes(self, mapping: &impl Fn(&str) -> Option<AccountAddress>) -> Result<Vec<u8>> {
        self.to_move_value(mapping)?
            .simple_serialize()
            .ok_or_else(|| anyhow!("Unable to serialize argument"))
    }
}

impl FromStr for FunctionArg {
    type Err = RoochError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Splits on the first colon, returning at most `2` elements
        // This is required to support args that contain a colon
        let parts: Vec<_> = s.splitn(2, ':').collect();
        let (ty, arg) = if parts.len() == 1 {
            // parse address @0x123 and unsigned integer 123u8
            if s.starts_with('@') {
                (FunctionArgType::Address, s.trim_start_matches('@'))
            } else {
                let u = s.splitn(2, 'u').collect::<Vec<_>>();
                if u.len() != 2 {
                    return Err(RoochError::CommandArgumentError(
                        "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_owned(),
                    ));
                } else {
                    let ty_str = String::from("u") + u[1];
                    let ty = FunctionArgType::from_str(&ty_str)?;
                    let arg = u[0];
                    (ty, arg)
                }
            }
        } else if parts.len() == 2 {
            let ty = FunctionArgType::from_str(parts[0])?;
            let arg = parts[1];
            (ty, arg)
        } else {
            return Err(RoochError::CommandArgumentError(
                "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_owned(),
            ));
        };
        let arg = ty.parse_arg(arg)?;

        Ok(arg)
    }
}

pub fn parse_function_arg(s: &str) -> Result<FunctionArg, RoochError> {
    // Splits on the first colon, returning at most `2` elements
    // This is required to support args that contain a colon
    let parts: Vec<_> = s.splitn(2, ':').collect();
    let (ty, arg) = if parts.len() == 1 {
        // parse address @0x123 and unsigned integer 123u8
        if s.starts_with('@') {
            (FunctionArgType::Address, s.trim_start_matches('@'))
        } else {
            let u = s.splitn(2, 'u').collect::<Vec<_>>();
            if u.len() != 2 {
                return Err(RoochError::CommandArgumentError(
                    "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_owned(),
                ));
            } else {
                let ty_str = String::from("u") + u[1];
                let ty = FunctionArgType::from_str(&ty_str)?;
                let arg = u[0];
                (ty, arg)
            }
        }
    } else if parts.len() == 2 {
        let ty = FunctionArgType::from_str(parts[0])?;
        let arg = parts[1];
        (ty, arg)
    } else {
        return Err(RoochError::CommandArgumentError(
            "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_owned(),
        ));
    };
    let arg = ty.parse_arg(arg)?;

    Ok(arg)
}

impl ParsableValue for FunctionArg {
    type ConcreteValue = MoveValue;

    fn parse_value<'a, I: Iterator<Item = (ValueToken, &'a str)>>(
        parser: &mut Parser<'a, ValueToken, I>,
    ) -> Option<anyhow::Result<Self>> {
        match parser.peek() {
            Some((ValueToken::Ident, arg_type_str)) => {
                match FunctionArgType::from_str(arg_type_str) {
                    Ok(arg_type) => {
                        //skp current token
                        parser.advance_any().unwrap();
                        if let Err(e) = parser.advance(ValueToken::Colon) {
                            return Some(Err(anyhow!("Expected colon, but got:{:?}", e)));
                        }
                        let arg = advance_all(parser);
                        Some(arg_type.parse_arg(&arg))
                    }
                    Err(_e) => None,
                }
            }
            _ => None,
        }
    }

    fn move_value_into_concrete(
        v: move_core_types::value::MoveValue,
    ) -> anyhow::Result<Self::ConcreteValue> {
        Ok(v)
    }

    fn concrete_vector(elems: Vec<Self::ConcreteValue>) -> anyhow::Result<Self::ConcreteValue> {
        Ok(MoveValue::Vector(elems))
    }

    fn concrete_struct(
        _addr: AccountAddress,
        _module: String,
        _name: String,
        values: std::collections::BTreeMap<String, Self::ConcreteValue>,
    ) -> anyhow::Result<Self::ConcreteValue> {
        Ok(MoveValue::Struct(MoveStruct::Runtime(
            values.into_values().collect(),
        )))
    }

    fn into_concrete_value(
        self,
        mapping: &impl Fn(&str) -> Option<AccountAddress>,
    ) -> anyhow::Result<Self::ConcreteValue> {
        self.to_move_value(mapping)
    }
}

fn advance_all<'a, I: Iterator<Item = (ValueToken, &'a str)>>(
    parser: &mut Parser<'a, ValueToken, I>,
) -> String {
    let mut s = String::new();
    while let Ok((_, arg)) = parser.advance_any() {
        s.push_str(arg);
    }
    s
}
