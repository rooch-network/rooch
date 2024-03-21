// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::raw_table;
use crate::natives::{
    helpers::make_module_natives, helpers::make_native,
    moveos_stdlib::raw_table::NativeTableContext,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{gas_algebra::InternalGas, vm_status::StatusCode};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use moveos_types::{
    moveos_std::{object::Object, object::ObjectID},
    state::{MoveState, PlaceholderStruct},
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

/// Ensure the error codes in this file is consistent with the error code in raw_table.move
pub(crate) const ERROR_ALREADY_EXISTS: u64 = 1;
pub(crate) const ERROR_NOT_FOUND: u64 = 2;
pub(crate) const ERROR_OBJECT_ALREADY_BORROWED: u64 = 7;
pub(crate) const ERROR_TYPE_MISMATCH: u64 = 10;

#[derive(Debug, Clone)]
pub struct AsRefGasParameters {
    pub base: InternalGas,
}

impl AsRefGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun as_ref_inner
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_as_ref_inner(
    gas_params: &AsRefGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.len() == 1);

    let object_id = arguments.pop_back().unwrap();
    let object_ref = borrow_object_reference(context, object_id, &ty_args[0])?;
    Ok(NativeResult::ok(gas_params.base, smallvec![object_ref]))
}

pub fn make_native_as_ref_inner(gas_params: AsRefGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| native_as_ref_inner(&gas_params, context, ty_args, args))
}

#[derive(Debug, Clone)]
pub struct AsMutRefGasParameters {
    pub base: InternalGas,
}

impl AsMutRefGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun as_mut_ref_inner
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_as_mut_ref_inner(
    gas_params: &AsMutRefGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.len() == 1);

    let object_id = arguments.pop_back().unwrap();
    let object_ref = borrow_object_reference(context, object_id, &ty_args[0])?;
    Ok(NativeResult::ok(gas_params.base, smallvec![object_ref]))
}

pub fn make_native_as_mut_ref_inner(gas_params: AsMutRefGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_as_mut_ref_inner(&gas_params, context, ty_args, args)
    })
}

fn borrow_object_reference(
    context: &mut NativeContext,
    object_id_value: Value,
    ref_type: &Type,
) -> PartialVMResult<Value> {
    let type_tag = context.type_to_type_tag(ref_type)?;
    let type_layout = context
        .get_type_layout(&type_tag)
        .map_err(|e| e.to_partial())?;

    debug_assert!(
        Object::<PlaceholderStruct>::type_layout_match(&type_layout),
        "Expected a struct type with layout same as Object<T>"
    );

    let object_id = ObjectID::from_runtime_value(object_id_value)
        .map_err(|_e| partial_extension_error("Invalid object id argument"))?;
    let table_context = context.extensions_mut().get_mut::<NativeTableContext>();

    let data = table_context.table_data();
    let mut table_data = data.write();
    //TODO remove load_object, the object should loaded when load ObjectEntity
    table_data
        .load_object(&object_id)
        .map_err(|e| e.to_partial())?;
    table_data
        .borrow_object(&object_id)
        .map_err(|e| e.to_partial())
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: raw_table::CommonGasParameters,
    pub as_ref_inner: AsRefGasParameters,
    pub as_mut_ref_inner: AsMutRefGasParameters,
    pub add_box: raw_table::AddBoxGasParameters,
    pub borrow_box: raw_table::BorrowBoxGasParameters,
    pub contains_box: raw_table::ContainsBoxGasParameters,
    pub contains_box_with_value_type: raw_table::ContainsBoxGasParameters,
    pub remove_box: raw_table::RemoveGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: raw_table::CommonGasParameters {
                load_base: 0.into(),
                load_per_byte: 0.into(),
                load_failure: 0.into(),
            },
            as_ref_inner: AsRefGasParameters::zeros(),
            as_mut_ref_inner: AsMutRefGasParameters::zeros(),
            add_box: raw_table::AddBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            borrow_box: raw_table::BorrowBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            contains_box: raw_table::ContainsBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            contains_box_with_value_type: raw_table::ContainsBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            remove_box: raw_table::RemoveGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "as_ref_inner",
            make_native(gas_params.as_ref_inner, native_as_ref_inner),
        ),
        (
            "as_mut_ref_inner",
            make_native(gas_params.as_mut_ref_inner, native_as_mut_ref_inner),
        ),
        (
            "add_box",
            raw_table::make_native_add_box(gas_params.common.clone(), gas_params.add_box),
        ),
        (
            "borrow_box",
            raw_table::make_native_borrow_box(
                gas_params.common.clone(),
                gas_params.borrow_box.clone(),
            ),
        ),
        (
            "borrow_box_mut",
            raw_table::make_native_borrow_box(gas_params.common.clone(), gas_params.borrow_box),
        ),
        (
            "remove_box",
            raw_table::make_native_remove_box(gas_params.common.clone(), gas_params.remove_box),
        ),
        (
            "contains_box",
            raw_table::make_native_contains_box(gas_params.common.clone(), gas_params.contains_box),
        ),
        (
            "contains_box_with_value_type",
            raw_table::make_native_contains_box_with_value_type(
                gas_params.common,
                gas_params.contains_box_with_value_type,
            ),
        ),
    ];

    make_module_natives(natives)
}

fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}
