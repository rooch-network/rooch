// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::object_field_fn::*;
use self::object_fn::*;
use self::object_meta_fn::*;
use crate::natives::helpers::make_module_natives;
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::values::Value;
use moveos_types::{
    moveos_std::object::{Object, ObjectID},
    state::{MoveState, PlaceholderStruct},
};
use std::collections::VecDeque;

pub use moveos_object_runtime::runtime::{
    ERROR_ALREADY_EXISTS, ERROR_NOT_FOUND, ERROR_OBJECT_ALREADY_BORROWED,
    ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED, ERROR_OBJECT_RUNTIME_ERROR, ERROR_TYPE_MISMATCH,
};

mod object_field_fn;
mod object_fn;
mod object_meta_fn;

#[derive(Debug, Clone)]
pub struct CommonGasParameters {
    pub load_base: InternalGas,
    pub load_per_byte: InternalGasPerByte,
    pub load_failure: InternalGas,
}

impl CommonGasParameters {
    fn calculate_load_cost(&self, loaded: Option<Option<NumBytes>>) -> InternalGas {
        self.load_base
            + match loaded {
                Some(Some(num_bytes)) => self.load_per_byte * num_bytes,
                Some(None) => self.load_failure,
                None => 0.into(),
            }
    }
}

// =========================================================================================
// Helpers

pub(crate) fn pop_object_id(args: &mut VecDeque<Value>) -> PartialVMResult<ObjectID> {
    let handle = args.pop_back().unwrap();
    ObjectID::from_runtime_value(handle).map_err(|e| {
        if log::log_enabled!(log::Level::Debug) {
            log::warn!("[ObjectRuntime] get_object_id: {:?}", e);
        }
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })
}

// Read ObjectID from Object<T> runtime value
pub(crate) fn read_object_id(value: &Value) -> PartialVMResult<ObjectID> {
    let obj = Object::<PlaceholderStruct>::from_runtime_value(value.copy_value()?)
        .map_err(|e| partial_extension_error(format!("Invalid object argument: {:?}", e)))?;
    Ok(obj.id)
}

pub(crate) fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    log::debug!("PartialVMError: {}", msg.to_string());
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}

pub(crate) fn error_to_abort_code(err: PartialVMError) -> u64 {
    //Because the PartialVMError do not provide function to get sub status code, we convert the error to VMError.
    let err = err.finish(Location::Undefined);

    let abort_code = match err.major_status() {
        StatusCode::MISSING_DATA => ERROR_NOT_FOUND,
        StatusCode::TYPE_MISMATCH => ERROR_TYPE_MISMATCH,
        StatusCode::RESOURCE_ALREADY_EXISTS => ERROR_ALREADY_EXISTS,
        StatusCode::ABORTED => err.sub_status().unwrap_or(ERROR_OBJECT_RUNTIME_ERROR),
        _ => ERROR_OBJECT_RUNTIME_ERROR,
    };
    if log::log_enabled!(log::Level::Debug) {
        log::warn!(
            "[ObjectRuntime] error err: {:?}, abort: {}",
            err,
            abort_code
        );
    };
    abort_code
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: CommonGasParameters,
    pub native_object_meta: ObjectMetaGasParameters,
    // Object functions
    pub native_borrow_object: BorrowObjectGasParameters,
    pub native_take_object: TakeObjectGasParameters,
    pub native_transfer_object: TransferObjectGasParameters,
    pub native_to_shared_object: ToSharedObjectGasParameters,
    pub native_to_frozen_object: ToFrozenObjectGasParameters,
    // Object field functions
    pub native_add_field: AddFieldGasParameters,
    pub native_borrow_field: BorrowFieldGasParameters,
    pub native_contains_field: ContainsFieldGasParameters,
    pub native_contains_field_with_value_type: ContainsFieldGasParameters,
    pub native_remove_field: RemoveFieldGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParameters {
                load_base: 0.into(),
                load_per_byte: 0.into(),
                load_failure: 0.into(),
            },
            native_object_meta: ObjectMetaGasParameters::zeros(),
            native_borrow_object: BorrowObjectGasParameters::zeros(),
            native_take_object: TakeObjectGasParameters::zeros(),
            native_transfer_object: TransferObjectGasParameters::zeros(),
            native_to_shared_object: ToSharedObjectGasParameters::zeros(),
            native_to_frozen_object: ToFrozenObjectGasParameters::zeros(),
            native_add_field: AddFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_borrow_field: BorrowFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_contains_field: ContainsFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_contains_field_with_value_type: ContainsFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_remove_field: RemoveFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "native_object_owner",
            make_native_object_owner(
                gas_params.common.clone(),
                gas_params.native_object_meta.clone(),
            ),
        ),
        (
            "native_object_size",
            make_native_object_size(
                gas_params.common.clone(),
                gas_params.native_object_meta.clone(),
            ),
        ),
        (
            "native_object_flag",
            make_native_object_flag(
                gas_params.common.clone(),
                gas_params.native_object_meta.clone(),
            ),
        ),
        (
            "native_object_created_at",
            make_native_object_created_at(
                gas_params.common.clone(),
                gas_params.native_object_meta.clone(),
            ),
        ),
        (
            "native_object_updated_at",
            make_native_object_updated_at(gas_params.common.clone(), gas_params.native_object_meta),
        ),
        (
            "native_borrow_object",
            make_native_borrow_object(
                gas_params.common.clone(),
                gas_params.native_borrow_object.clone(),
            ),
        ),
        (
            "native_borrow_mut_object",
            make_native_borrow_object(gas_params.common.clone(), gas_params.native_borrow_object),
        ),
        (
            "native_take_object",
            make_native_take_object(gas_params.common.clone(), gas_params.native_take_object),
        ),
        (
            "native_transfer_object",
            make_native_transfer_object(
                gas_params.common.clone(),
                gas_params.native_transfer_object,
            ),
        ),
        (
            "native_to_shared_object",
            make_native_to_shared_object(
                gas_params.common.clone(),
                gas_params.native_to_shared_object,
            ),
        ),
        (
            "native_to_frozen_object",
            make_native_to_frozen_object(
                gas_params.common.clone(),
                gas_params.native_to_frozen_object,
            ),
        ),
        (
            "native_add_field",
            make_native_add_field(gas_params.common.clone(), gas_params.native_add_field),
        ),
        (
            "native_borrow_field",
            make_native_borrow_field(
                gas_params.common.clone(),
                gas_params.native_borrow_field.clone(),
            ),
        ),
        (
            "native_borrow_mut_field",
            make_native_borrow_field(gas_params.common.clone(), gas_params.native_borrow_field),
        ),
        (
            "native_remove_field",
            make_native_remove_field(gas_params.common.clone(), gas_params.native_remove_field),
        ),
        (
            "native_contains_field",
            make_native_contains_field(gas_params.common.clone(), gas_params.native_contains_field),
        ),
        (
            "native_contains_field_with_value_type",
            make_native_contains_field_with_value_type(
                gas_params.common,
                gas_params.native_contains_field_with_value_type,
            ),
        ),
    ];

    make_module_natives(natives)
}
