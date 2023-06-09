// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::language_storage::TypeTag;
use move_resource_viewer::AnnotatedMoveValue;
use serde::{Deserialize, Serialize};

/// The function return value in MoveOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionReturnValue {
    pub type_tag: TypeTag,
    pub value: Vec<u8>,
}

impl FunctionReturnValue {
    pub fn new(type_tag: TypeTag, value: Vec<u8>) -> Self {
        Self { type_tag, value }
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedFunctionReturnValue {
    pub value: FunctionReturnValue,
    pub move_value: AnnotatedMoveValue,
}
