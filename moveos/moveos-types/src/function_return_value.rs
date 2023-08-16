// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{VMError, VMResult};
use move_core_types::{language_storage::TypeTag, vm_status::VMStatus};
use move_resource_viewer::AnnotatedMoveValue;
use serde::{Deserialize, Serialize};

/// The result of a readonly function call in MoveOS
/// If the vm_status is not Executed, the return_values will be None
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    pub vm_status: VMStatus,
    pub return_values: Option<Vec<FunctionReturnValue>>,
}

impl FunctionResult {
    pub fn ok(return_values: Vec<FunctionReturnValue>) -> Self {
        Self {
            vm_status: VMStatus::Executed,
            return_values: Some(return_values),
        }
    }

    pub fn err(vm_error: VMError) -> Self {
        Self {
            vm_status: vm_error.into_vm_status(),
            return_values: None,
        }
    }

    pub fn into_result(self) -> Result<Vec<FunctionReturnValue>, anyhow::Error> {
        match self.vm_status {
            VMStatus::Executed => Ok(self
                .return_values
                .expect("return_values must be Some, if vm_status is Executed")),
            _ => Err(anyhow::anyhow!(
                "Function call failed with VMStatus: {:?}",
                self.vm_status
            )),
        }
    }
}

impl From<VMResult<Vec<FunctionReturnValue>>> for FunctionResult {
    fn from(result: VMResult<Vec<FunctionReturnValue>>) -> Self {
        match result {
            Ok(return_values) => Self::ok(return_values),
            Err(vm_error) => Self {
                vm_status: vm_error.into_vm_status(),
                return_values: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedFunctionResult {
    pub vm_status: VMStatus,
    pub return_values: Option<Vec<AnnotatedFunctionReturnValue>>,
}

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
