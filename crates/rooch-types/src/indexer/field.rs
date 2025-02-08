// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::effects::Op;
use move_core_types::language_storage::TypeTag;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::{
    is_dynamic_field_type, is_field_struct_tag, ObjectID, ObjectMeta, RawField,
};
use moveos_types::state::{FieldKey, ObjectChange};
use serde::{Deserialize, Serialize};

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
    pub fn new(metadata: ObjectMeta, field_key: FieldKey, value: u64) -> Self {
        IndexerField {
            id: metadata.id,
            field_key: field_key.to_hex_literal(),
            name: "".to_string(), //for expansion
            value,

            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        }
    }
}

// #[derive(
//     Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
// )]
// pub struct IndexerFieldID {
//     pub object_id: ObjectID,
//     pub field_key: String,
// }
//
// impl std::fmt::Display for IndexerFieldID {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "IndexerFieldID[object id: {:?}, field key: {}]",
//             self.object_id, self.field_key,
//         )
//     }
// }
//
// impl IndexerFieldID {
//     pub fn new(object_id: ObjectID, field_key: String) -> Self {
//         IndexerFieldID {
//             object_id,
//             field_key,
//         }
//     }
// }

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
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();

    // TODO check field index config
    // index dynamic field object
    if is_dynamic_field_type(&object_type) {
        let name_and_value_typetag_opt = parse_dynamic_field_type_tags(&object_type);
        if let Some((name_type, value_type)) = name_and_value_typetag_opt {
            if let Some(op) = value {
                match op {
                    Op::Modify(field_value) => {
                        let raw_field = RawField::parse_unchecked_field(
                            field_value.as_slice(),
                            name_type,
                            value_type,
                        )?;
                        let origin_value_opt =
                            resolve_value_to_u64(&raw_field.value_type, raw_field.value);
                        if let Some(origin_value) = origin_value_opt {
                            let field =
                                IndexerField::new(metadata.clone(), field_key, origin_value);
                            field_changes.update_fields.push(field);
                        }
                    }
                    Op::Delete => {
                        field_changes
                            .remove_fields
                            .push((object_id.clone().to_string(), field_key.to_hex_literal()));
                    }
                    Op::New(field_value) => {
                        let raw_field = RawField::parse_unchecked_field(
                            field_value.as_slice(),
                            name_type,
                            value_type,
                        )?;
                        let origin_value_opt =
                            resolve_value_to_u64(&raw_field.value_type, raw_field.value);
                        if let Some(origin_value) = origin_value_opt {
                            let field =
                                IndexerField::new(metadata.clone(), field_key, origin_value);
                            field_changes.new_fields.push(field);
                        }
                    }
                }
            }
        }

        return Ok(());
    }

    for (key, change) in fields {
        handle_field_change(key, change, field_changes)?;
    }
    Ok(())
}

pub fn parse_dynamic_field_type_tags(type_tag: &TypeTag) -> Option<(TypeTag, TypeTag)> {
    if let TypeTag::Struct(struct_tag) = type_tag {
        // Verify this is a DynamicField struct
        if is_field_struct_tag(struct_tag) {
            // DynamicField should have exactly 2 type parameters
            if struct_tag.type_params.len() == 2 {
                // Get Name and Value type tags
                let name_type = struct_tag.type_params[0].clone();
                let value_type = struct_tag.type_params[1].clone();
                return Some((name_type, value_type));
            }
        }
    }
    None
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

//
// pub fn handle_revert_object_change(
//     state_index_generator: &mut IndexerFieldsIndexGenerator,
//     tx_order: u64,
//     indexer_object_state_change_set: &mut IndexerObjectStateChangeSet,
//     object_change: ObjectChange,
//     object_mapping: &HashMap<ObjectID, ObjectMeta>,
// ) -> Result<()> {
//     let ObjectChange {
//         metadata,
//         value,
//         fields,
//     } = object_change;
//     let object_id = metadata.id.clone();
//     let object_type = metadata.object_type.clone();
//     let state_index = state_index_generator.get(&object_type);
//
//     // Do not index dynamic field object
//     if is_dynamic_field_type(&object_type) {
//         return Ok(());
//     }
//     if let Some(op) = value {
//         match op {
//             Op::Modify(_value) => {
//                 // Keep the tx_order and state index consistent before reverting
//                 if let Some(previous_object_meta) = object_mapping.get(&object_id) {
//                     let state = IndexerObjectState::new(
//                         previous_object_meta.clone(),
//                         tx_order,
//                         state_index,
//                     );
//                     indexer_object_state_change_set.update_fields(state);
//                 }
//             }
//             Op::Delete => {
//                 // Use the reverted tx_order and state index as the deleted restored tx_order and tx_order
//                 if let Some(previous_object_meta) = object_mapping.get(&object_id) {
//                     let state = IndexerObjectState::new(
//                         previous_object_meta.clone(),
//                         tx_order,
//                         state_index,
//                     );
//                     indexer_object_state_change_set.new_fields(state);
//                 }
//             }
//             Op::New(_value) => {
//                 indexer_object_state_change_set.remove_fields(object_id, &object_type);
//             }
//         }
//     }
//
//     state_index_generator.incr(&object_type);
//     for (_key, change) in fields {
//         handle_revert_object_change(
//             state_index_generator,
//             tx_order,
//             indexer_object_state_change_set,
//             change,
//             object_mapping,
//         )?;
//     }
//     Ok(())
// }
//
// pub fn collect_revert_object_change_ids(
//     object_change: ObjectChange,
//     object_ids: &mut Vec<ObjectID>,
// ) -> Result<()> {
//     let ObjectChange {
//         metadata,
//         value,
//         fields,
//     } = object_change;
//     let object_id = metadata.id.clone();
//     let object_type = metadata.object_type.clone();
//
//     // Do not index dynamic field object
//     if is_dynamic_field_type(&object_type) {
//         return Ok(());
//     }
//     if let Some(op) = value {
//         match op {
//             Op::Modify(_value) => {
//                 object_ids.push(object_id);
//             }
//             Op::Delete => {
//                 object_ids.push(object_id);
//             }
//             Op::New(_value) => {}
//         }
//     }
//
//     for (_key, change) in fields {
//         collect_revert_object_change_ids(change, object_ids)?;
//     }
//     Ok(())
// }
//
