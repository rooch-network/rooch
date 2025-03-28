// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::AnnotatedMoveValueView;
use anyhow::Result;
use move_core_types::language_storage::StructTag;
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use moveos_types::{
    move_std::option::MoveOption,
    state::{MoveStructType, PlaceholderStruct},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, JsonSchema, Eq, PartialEq)]
pub struct MoveOptionView(Option<AnnotatedMoveValueView>);

impl MoveOptionView {
    pub fn as_ref(&self) -> Option<&AnnotatedMoveValueView> {
        self.0.as_ref()
    }

    pub fn struct_tag_match(struct_tag: &StructTag) -> bool {
        MoveOption::<PlaceholderStruct>::struct_tag_match_without_type_param(struct_tag)
    }
}

impl TryFrom<&AnnotatedMoveStruct> for MoveOptionView {
    type Error = anyhow::Error;

    fn try_from(annotated_move_struct: &AnnotatedMoveStruct) -> Result<Self, Self::Error> {
        let mut fields = annotated_move_struct.value.iter();
        let (field_name, field_value) = fields
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid DecimalValue"))?;
        debug_assert!(field_name.as_str() == "vec");

        match field_value {
            AnnotatedMoveValue::Vector(_type, data) => {
                if data.is_empty() {
                    Ok(MoveOptionView(None))
                } else {
                    if data.len() > 1 {
                        // This is unexpected for an Option type - log or handle appropriately
                        return Err(anyhow::anyhow!("Option vector has more than one element"));
                    }
                    Ok(MoveOptionView(Some(AnnotatedMoveValueView::from(
                        data.first().unwrap().clone(),
                    ))))
                }
            }
            _ => Err(anyhow::anyhow!("Invalid Option")),
        }
    }
}
