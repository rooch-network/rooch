// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

// use super::object::Object;
use super::object::{self, ObjectID};
use crate::{
    addresses::{MOVEOS_STD_ADDRESS, MOVE_STD_ADDRESS},
    move_std::string::MoveString,
    moveos_std::simple_map::SimpleMap,
};
use anyhow::{self, Result};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("display");

pub fn get_object_display_id(value_type: TypeTag) -> ObjectID {
    let object_tag = StructTag {
        address: MOVEOS_STD_ADDRESS,
        name: ident_str!("Object").to_owned(),
        module: ident_str!("object").to_owned(),
        type_params: vec![value_type],
    };
    let object_type = TypeTag::from(object_tag);
    let struct_tag = StructTag {
        address: MOVEOS_STD_ADDRESS,
        name: ident_str!("Display").to_owned(),
        module: MODULE_NAME.to_owned(),
        type_params: vec![object_type],
    };
    object::named_object_id(&struct_tag)
}

pub fn get_resource_display_id(value_type: TypeTag) -> ObjectID {
    let struct_tag = StructTag {
        address: MOVEOS_STD_ADDRESS,
        name: ident_str!("Display").to_owned(),
        module: MODULE_NAME.to_owned(),
        type_params: vec![value_type],
    };
    object::named_object_id(&struct_tag)
}

/// Convert 0x1::string::String to displayable string.
fn display_move_string(move_struct: &AnnotatedMoveStruct) -> Result<String> {
    if let AnnotatedMoveValue::Bytes(bytes) = &move_struct.value[0].1 {
        String::from_utf8(bytes.clone()).map_err(|e| e.into())
    } else {
        unreachable!("Invalid move string type")
    }
}

/// Convert 0x2::object::ObjectID to displayable string.
fn display_object_id(move_struct: &AnnotatedMoveStruct) -> String {
    if let AnnotatedMoveValue::Address(address) = &move_struct.value[0].1 {
        address.to_canonical_string()
    } else {
        unreachable!("Invalid object_id type")
    }
}

fn get_string_from_valid_move_struct(move_struct: &AnnotatedMoveStruct) -> Result<String> {
    let move_std_string = StructTag {
        address: MOVE_STD_ADDRESS,
        module: ident_str!("string").to_owned(),
        name: ident_str!("String").to_owned(),
        type_params: vec![],
    };
    let moveos_std_object_id = StructTag {
        address: MOVEOS_STD_ADDRESS,
        module: ident_str!("object_id").to_owned(),
        name: ident_str!("ObjectID").to_owned(),
        type_params: vec![],
    };

    if move_struct.type_ == move_std_string {
        display_move_string(move_struct)
    } else if move_struct.type_ == moveos_std_object_id {
        Ok(display_object_id(move_struct))
    } else {
        anyhow::bail!("Invalid move type to display");
    }
}

fn get_value_from_move_struct(move_value: &AnnotatedMoveValue, var_name: &str) -> Result<String> {
    let parts: Vec<&str> = var_name.split('.').collect();
    if parts.is_empty() {
        anyhow::bail!("Display template value cannot be empty");
    }
    let mut current_value = move_value;
    // iterate over the parts and try to access the corresponding field
    for part in parts {
        match current_value {
            AnnotatedMoveValue::Struct(move_struct) => {
                let mut fields = BTreeMap::new();
                for (key, value) in move_struct.value.iter() {
                    fields.insert(key.to_string(), value);
                }
                if let Some(value) = fields.get(&part.to_owned()) {
                    current_value = value;
                } else {
                    anyhow::bail!("Field value {} cannot be found in struct", var_name);
                }
            }
            _ => {
                anyhow::bail!("Unexpected move value type for field {}", var_name);
            }
        }
    }

    match current_value {
        AnnotatedMoveValue::Vector(_, _) | AnnotatedMoveValue::Bytes(_) => {
            anyhow::bail!(
                "Vector or bytes are not supported as a Display value {}",
                var_name
            );
        }
        AnnotatedMoveValue::Struct(last_field) => get_string_from_valid_move_struct(last_field),
        _ => Ok(current_value.to_string()),
    }
}

fn parse_template(template: &str, move_value: &AnnotatedMoveValue) -> Result<String> {
    let mut output = template.to_string();
    let mut var_name = String::new();
    let mut in_braces = false;
    let mut escaped = false;

    for ch in template.chars() {
        match ch {
            '{' if !escaped => {
                in_braces = true;
                var_name.clear();
            }
            '}' if !escaped => {
                in_braces = false;
                let value = get_value_from_move_struct(move_value, &var_name)?;
                output = output.replace(&format!("{{{}}}", var_name), &value);
            }
            _ if !escaped => {
                if in_braces {
                    var_name.push(ch);
                }
            }
            _ => {}
        }
        escaped = false;
    }
    Ok(output)
}

/// Display struct in rust, binding for moveos_std::display::Display
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct RawDisplay {
    pub fields: SimpleMap<MoveString, MoveString>,
}

impl RawDisplay {
    /// Render the display with given MoveStruct instance.
    // pub fn render(&self, annotated_obj: &AnnotatedObject) -> BTreeMap<String, String> {
    pub fn render(&self, annotated_obj: &AnnotatedMoveValue) -> BTreeMap<String, String> {
        let fields = self.to_btree_map().into_iter().map(|entry| {
            match parse_template(&entry.1, annotated_obj) {
                Ok(value) => (entry.0, value),
                Err(err) => {
                    println!("Display template render error: {:?}", err);
                    entry // TODO: handle render error
                }
            }
        });
        BTreeMap::from_iter(fields)
    }

    pub fn to_btree_map(&self) -> BTreeMap<String, String> {
        let mut btree_map = BTreeMap::new();
        for element in &self.fields.data {
            btree_map.insert(element.key.to_string(), element.value.to_string());
        }
        btree_map
    }
}
