// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use anyhow::{anyhow, Result};
use move_core_types::effects::Op;
use move_core_types::language_storage::TypeTag;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::{
    get_bcs_slice, is_dynamic_field_type, parse_dynamic_field_type_tags, ObjectID, ObjectMeta,
    RawField,
};
use moveos_types::state::{FieldKey, ObjectChange, ObjectState};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct IndexerField {
    pub id: ObjectID,
    pub field_key: String,
    pub name: String,
    pub value: u64,
    /// the field item created timestamp on chain
    pub created_at: u64,
    /// the field item updated timestamp on chain
    pub updated_at: u64,
}

impl IndexerField {
    pub fn new(metadata: ObjectMeta, field_key: FieldKey, name: String, value: u64) -> Self {
        IndexerField {
            id: metadata.id.parent().unwrap_or(ObjectID::root()),
            field_key: field_key.to_hex_literal(),
            name,
            value,

            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldFilter {
    /// Query by object ids.
    ObjectId(Vec<ObjectID>),
}

impl FieldFilter {
    fn try_matches(&self, item: &IndexerField) -> Result<bool> {
        Ok(match self {
            FieldFilter::ObjectId(object_ids) => object_ids.len() == 1 && object_ids[0] == item.id,
        })
    }
}

impl Filter<IndexerField> for FieldFilter {
    fn matches(&self, item: &IndexerField) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IndexerFieldChanges {
    pub new_fields: Vec<IndexerField>,
    pub update_fields: Vec<IndexerField>,
    pub remove_fields: Vec<(String, String)>,
    pub remove_fields_by_id: Vec<String>,
}

pub fn handle_field_change(
    field_key: FieldKey,
    object_change: ObjectChange,
    field_changes: &mut IndexerFieldChanges,
    field_indexer_ids: &Vec<ObjectID>,
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();

    // first, check field index config
    // then, index dynamic field object
    if need_process_field_indexer(&object_id, field_indexer_ids)
        && is_dynamic_field_type(&object_type)
    {
        let name_and_value_typetag_opt = parse_dynamic_field_type_tags(&object_type);
        if let Some((name_type, value_type)) = name_and_value_typetag_opt {
            if let Some(op) = value {
                match op {
                    Op::Modify(field_value) => {
                        // ignore dynamic raw field parse error
                        let raw_field_opt = RawField::parse_unchecked_field(
                            field_value.as_slice(),
                            name_type,
                            value_type,
                        )
                        .ok();

                        if let Some(raw_field) = raw_field_opt {
                            let origin_name =
                                bytes_to_string(raw_field.name.as_slice(), &raw_field.name_type)
                                    .unwrap_or("".to_string());
                            let origin_value_opt =
                                resolve_value_to_u64(&raw_field.value_type, raw_field.value);
                            if let Some(origin_value) = origin_value_opt {
                                let field = IndexerField::new(
                                    metadata.clone(),
                                    field_key,
                                    origin_name,
                                    origin_value,
                                );
                                field_changes.update_fields.push(field);
                            }
                        }
                    }
                    Op::Delete => {
                        field_changes.remove_fields.push((
                            object_id
                                .clone()
                                .parent()
                                .unwrap_or(ObjectID::root())
                                .to_string(),
                            field_key.to_hex_literal(),
                        ));
                    }
                    Op::New(field_value) => {
                        // ignore dynamic raw field parse error
                        let raw_field_opt = RawField::parse_unchecked_field(
                            field_value.as_slice(),
                            name_type,
                            value_type,
                        )
                        .ok();

                        if let Some(raw_field) = raw_field_opt {
                            let origin_name =
                                bytes_to_string(raw_field.name.as_slice(), &raw_field.name_type)
                                    .unwrap_or("".to_string());
                            let origin_value_opt =
                                resolve_value_to_u64(&raw_field.value_type, raw_field.value);
                            if let Some(origin_value) = origin_value_opt {
                                let field = IndexerField::new(
                                    metadata.clone(),
                                    field_key,
                                    origin_name,
                                    origin_value,
                                );
                                field_changes.new_fields.push(field);
                            }
                        }
                    }
                }
            }
        }

        return Ok(());
    }

    for (key, change) in fields {
        handle_field_change(key, change, field_changes, field_indexer_ids)?;
    }
    Ok(())
}

pub fn need_process_field_indexer(id: &ObjectID, field_indexer_ids: &[ObjectID]) -> bool {
    if let Some(parent) = id.parent() {
        field_indexer_ids.contains(&parent)
    } else {
        false
    }
}

pub fn resolve_value_to_u64(type_tag: &TypeTag, value: Vec<u8>) -> Option<u64> {
    match type_tag {
        TypeTag::U8 => bcs::from_bytes::<u8>(&value).ok().map(|v| v as u64),
        TypeTag::U16 => bcs::from_bytes::<u16>(&value).ok().map(|v| v as u64),
        TypeTag::U32 => bcs::from_bytes::<u32>(&value).ok().map(|v| v as u64),
        TypeTag::U64 => bcs::from_bytes::<u64>(&value).ok(),
        TypeTag::U128 => {
            // Handle potential overflow
            bcs::from_bytes::<u128>(&value).ok().and_then(|v| {
                if v <= u64::MAX as u128 {
                    Some(v as u64)
                } else {
                    None
                }
            })
        }
        TypeTag::U256 => {
            // Handle potential overflow
            bcs::from_bytes::<U256>(&value).ok().and_then(|v| {
                if v <= U256::from(u64::MAX) {
                    Some(v.unchecked_as_u64())
                } else {
                    None
                }
            })
        }
        _ => None,
    }
}

pub fn handle_revert_field_change(
    field_key: FieldKey,
    object_change: ObjectChange,
    field_changes: &mut IndexerFieldChanges,
    field_indexer_ids: &Vec<ObjectID>,
    field_object_mapping: &HashMap<ObjectID, ObjectState>,
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;

    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();

    if need_process_field_indexer(&object_id, field_indexer_ids)
        && is_dynamic_field_type(&object_type)
    {
        let name_and_value_typetag_opt = parse_dynamic_field_type_tags(&object_type);
        if let Some((name_type, value_type)) = name_and_value_typetag_opt {
            if let Some(op) = value {
                match op {
                    Op::Modify(_field_value) => {
                        if let Some(previous_field_object) = field_object_mapping.get(&object_id) {
                            // ignore dynamic raw field parse error
                            let raw_field_opt = RawField::parse_unchecked_field(
                                previous_field_object.value.as_slice(),
                                name_type,
                                value_type,
                            )
                            .ok();

                            if let Some(raw_field) = raw_field_opt {
                                let origin_name = bytes_to_string(
                                    raw_field.name.as_slice(),
                                    &raw_field.name_type,
                                )
                                .unwrap_or("".to_string());
                                let origin_value_opt =
                                    resolve_value_to_u64(&raw_field.value_type, raw_field.value);
                                if let Some(origin_value) = origin_value_opt {
                                    let field = IndexerField::new(
                                        previous_field_object.metadata.clone(),
                                        field_key,
                                        origin_name,
                                        origin_value,
                                    );
                                    field_changes.update_fields.push(field);
                                }
                            }
                        }
                    }
                    Op::Delete => {
                        if let Some(previous_field_object) = field_object_mapping.get(&object_id) {
                            // ignore dynamic raw field parse error
                            let raw_field_opt = RawField::parse_unchecked_field(
                                previous_field_object.value.as_slice(),
                                name_type,
                                value_type,
                            )
                            .ok();
                            if let Some(raw_field) = raw_field_opt {
                                let origin_value_opt =
                                    resolve_value_to_u64(&raw_field.value_type, raw_field.value);
                                let origin_name = bytes_to_string(
                                    raw_field.name.as_slice(),
                                    &raw_field.name_type,
                                )
                                .unwrap_or("".to_string());
                                if let Some(origin_value) = origin_value_opt {
                                    let field = IndexerField::new(
                                        previous_field_object.metadata.clone(),
                                        field_key,
                                        origin_name,
                                        origin_value,
                                    );
                                    field_changes.new_fields.push(field);
                                }
                            }
                        }
                    }
                    Op::New(_field_value) => {
                        field_changes.remove_fields.push((
                            object_id
                                .clone()
                                .parent()
                                .unwrap_or(ObjectID::root())
                                .to_string(),
                            field_key.to_hex_literal(),
                        ));
                    }
                }
            }
        }

        return Ok(());
    }

    for (key, change) in fields {
        handle_revert_field_change(
            key,
            change,
            field_changes,
            field_indexer_ids,
            field_object_mapping,
        )?;
    }
    Ok(())
}
//
pub fn collect_revert_field_change_ids(
    field_indexer_ids: &Vec<ObjectID>,
    object_change: ObjectChange,
    field_object_ids: &mut Vec<ObjectID>,
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();

    if need_process_field_indexer(&object_id, field_indexer_ids)
        && is_dynamic_field_type(&object_type)
    {
        if let Some(op) = value {
            match op {
                Op::Modify(_value) => {
                    field_object_ids.push(object_id);
                }
                Op::Delete => {
                    field_object_ids.push(object_id);
                }
                Op::New(_value) => {}
            }
        }
    }

    for (_key, change) in fields {
        collect_revert_field_change_ids(field_indexer_ids, change, field_object_ids)?;
    }
    Ok(())
}

pub fn bytes_to_json(bytes: &[u8], type_tag: &TypeTag) -> Result<JsonValue> {
    match type_tag {
        // Primitive types
        TypeTag::Bool => {
            let value: bool = bcs::from_bytes(bytes)?;
            Ok(JsonValue::Bool(value))
        }
        TypeTag::U8 => {
            let value: u8 = bcs::from_bytes(bytes)?;
            Ok(JsonValue::Number(value.into()))
        }
        TypeTag::U16 => {
            let value: u16 = bcs::from_bytes(bytes)?;
            Ok(JsonValue::Number(value.into()))
        }
        TypeTag::U32 => {
            let value: u32 = bcs::from_bytes(bytes)?;
            Ok(JsonValue::Number(value.into()))
        }
        TypeTag::U64 => {
            let value: u64 = bcs::from_bytes(bytes)?;
            Ok(JsonValue::String(value.to_string())) // Use string for u64 to avoid precision loss
        }
        TypeTag::U128 => {
            let value: u128 = bcs::from_bytes(bytes)?;
            Ok(JsonValue::String(value.to_string()))
        }
        TypeTag::U256 => {
            let value: [u8; 32] = bcs::from_bytes(bytes)?;
            Ok(JsonValue::String(format!("0x{}", hex::encode(value))))
        }
        TypeTag::Address => {
            let value: [u8; 32] = bcs::from_bytes(bytes)?;
            Ok(JsonValue::String(format!("0x{}", hex::encode(value))))
        }

        // Vector types
        TypeTag::Vector(elem_type) => {
            match &**elem_type {
                TypeTag::U8 => {
                    // Special case for vector<u8> - treat as string or hex
                    let bytes: Vec<u8> = bcs::from_bytes(bytes)?;
                    if bytes.iter().all(|b| b.is_ascii()) {
                        // If ASCII printable, convert to string
                        Ok(JsonValue::String(
                            String::from_utf8_lossy(&bytes).into_owned(),
                        ))
                    } else {
                        // Otherwise, convert to hex
                        Ok(JsonValue::String(format!("0x{}", hex::encode(&bytes))))
                    }
                }
                _ => {
                    // For other vector types, parse as array
                    let (length, prefix_size) = parse_uleb128(bytes)?;
                    let mut values = Vec::new();
                    let mut offset = prefix_size;

                    for _ in 0..length {
                        let (value_data, _next_offset) =
                            get_bcs_slice(&bytes[offset..], elem_type)?;
                        let value = bytes_to_json(&value_data, elem_type)?;
                        values.push(value);
                        offset += value_data.len();
                    }

                    Ok(JsonValue::Array(values))
                }
            }
        }

        // Struct types
        TypeTag::Struct(_struct_tag) => {
            // For structs, it need the resolver to get field information
            // todo!("Implement struct conversion for {}", struct_tag)
            Err(anyhow!("Unsupported type tag for struct: {:?}", type_tag))
        }

        _ => Err(anyhow!("Unsupported type tag: {:?}", type_tag)),
    }
}

// Helper function to get a string representation
pub fn bytes_to_string(bytes: &[u8], type_tag: &TypeTag) -> Result<String> {
    let json = bytes_to_json(bytes, type_tag)?;
    match json {
        JsonValue::String(s) => Ok(s),
        _ => Ok(json.to_string()),
    }
}

// Helper function to parse ULEB128-encoded length
pub fn parse_uleb128(bytes: &[u8]) -> Result<(usize, usize)> {
    let mut length: usize = 0;
    let mut shift = 0;
    let mut position = 0;

    loop {
        if position >= bytes.len() {
            return Err(anyhow!("Invalid ULEB128 encoding"));
        }

        let byte = bytes[position];
        length |= ((byte & 0x7f) as usize) << shift;
        position += 1;

        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    Ok((length, position))
}
