// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{AbortLocationView, BytesView};
use crate::jsonrpc_types::{
    move_types::{AnnotatedMoveValueView, TypeTagView},
    StrView,
};
use move_core_types::vm_status::{StatusCode, VMStatus};
use moveos_types::function_return_value::{
    AnnotatedFunctionResult, AnnotatedFunctionReturnValue, FunctionResult, FunctionReturnValue,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VMStatusView {
    Executed,
    MoveAbort {
        location: AbortLocationView,
        abort_code: StrView<u64>,
    },
    ExecutionFailure {
        location: AbortLocationView,
        function: u16,
        code_offset: u16,
        status_code: StrView<u64>,
    },
    Error(StrView<u64>),
}

impl From<VMStatus> for VMStatusView {
    fn from(vm_status: VMStatus) -> Self {
        match vm_status {
            VMStatus::Executed => Self::Executed,
            VMStatus::MoveAbort(location, abort_code) => Self::MoveAbort {
                location: location.into(),
                abort_code: StrView(abort_code),
            },
            VMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
                status_code,
                ..
            } => Self::ExecutionFailure {
                location: location.into(),
                function,
                code_offset,
                status_code: StrView(status_code as u64),
            },
            VMStatus::Error{status_code, ..} => Self::Error(StrView(status_code as u64)),
        }
    }
}

impl TryFrom<VMStatusView> for VMStatus {
    type Error = anyhow::Error;

    fn try_from(value: VMStatusView) -> Result<Self, anyhow::Error> {
        match value {
            VMStatusView::Executed => Ok(VMStatus::Executed),
            VMStatusView::MoveAbort {
                location,
                abort_code,
            } => Ok(VMStatus::MoveAbort(location.0, abort_code.0)),
            VMStatusView::ExecutionFailure {
                location,
                function,
                code_offset,
                status_code,
            } => Ok(VMStatus::ExecutionFailure {
                location: location.0,
                function,
                code_offset,
                status_code: StatusCode::try_from(status_code.0)
                    .map_err(|e| anyhow::anyhow!("StatusCode convert error:{}", e))?,
                sub_status: None,
                message: None,
            }),
            VMStatusView::Error(status_code) => Ok(VMStatus::Error {
                status_code: StatusCode::try_from(status_code.0)
                    .map_err(|e| anyhow::anyhow!("StatusCode convert error:{}", e))?,
                sub_status: None,
                message: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnnotatedFunctionResultView {
    pub vm_status: VMStatusView,
    pub return_values: Option<Vec<AnnotatedFunctionReturnValueView>>,
}

impl From<AnnotatedFunctionResult> for AnnotatedFunctionResultView {
    fn from(value: AnnotatedFunctionResult) -> Self {
        Self {
            vm_status: value.vm_status.into(),
            return_values: value
                .return_values
                .map(|v| v.into_iter().map(|v| v.into()).collect()),
        }
    }
}

impl TryFrom<AnnotatedFunctionResultView> for FunctionResult {
    type Error = anyhow::Error;

    fn try_from(value: AnnotatedFunctionResultView) -> Result<Self, Self::Error> {
        Ok(Self {
            vm_status: value.vm_status.try_into()?,
            return_values: value.return_values.map(|v| {
                v.into_iter()
                    .map(|v| v.value.into())
                    .collect::<Vec<FunctionReturnValue>>()
            }),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionReturnValueView {
    pub type_tag: TypeTagView,
    pub value: BytesView,
}

impl From<FunctionReturnValue> for FunctionReturnValueView {
    fn from(value: FunctionReturnValue) -> Self {
        Self {
            type_tag: value.type_tag.into(),
            value: StrView(value.value),
        }
    }
}

impl From<FunctionReturnValueView> for FunctionReturnValue {
    fn from(value: FunctionReturnValueView) -> Self {
        Self {
            type_tag: value.type_tag.into(),
            value: value.value.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnnotatedFunctionReturnValueView {
    pub value: FunctionReturnValueView,
    pub decoded_value: AnnotatedMoveValueView,
}

impl From<AnnotatedFunctionReturnValue> for AnnotatedFunctionReturnValueView {
    fn from(value: AnnotatedFunctionReturnValue) -> Self {
        Self {
            value: value.value.into(),
            decoded_value: value.decoded_value.into(),
        }
    }
}
