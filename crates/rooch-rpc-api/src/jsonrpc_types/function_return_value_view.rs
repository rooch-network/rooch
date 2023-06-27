// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    move_types::{AnnotatedMoveValueView, TypeTagView},
    StrView,
};
use moveos_types::function_return_value::{AnnotatedFunctionReturnValue, FunctionReturnValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionReturnValueView {
    pub type_tag: TypeTagView,
    pub value: StrView<Vec<u8>>,
}

impl From<FunctionReturnValue> for FunctionReturnValueView {
    fn from(value: FunctionReturnValue) -> Self {
        Self {
            type_tag: value.type_tag.into(),
            value: StrView(value.value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnnotatedFunctionReturnValueView {
    pub value: FunctionReturnValueView,
    pub move_value: AnnotatedMoveValueView,
}

impl From<AnnotatedFunctionReturnValue> for AnnotatedFunctionReturnValueView {
    fn from(value: AnnotatedFunctionReturnValue) -> Self {
        Self {
            value: value.value.into(),
            move_value: value.move_value.into(),
        }
    }
}
