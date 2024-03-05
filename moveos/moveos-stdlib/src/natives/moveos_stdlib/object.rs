// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::{helpers::make_module_natives, moveos_stdlib::raw_table::NativeTableContext};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{gas_algebra::InternalGas, vm_status::StatusCode};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use moveos_types::{
    moveos_std::{object::Object, object_id::ObjectID},
    state::{MoveState, PlaceholderStruct},
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

const ERROR_OBJECT_ALREADY_BORROWED: u64 = 7;

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
    let gv = table_data.get_or_create_object_reference(object_id)?;

    if gv.reference_count() >= 2 {
        return Err(PartialVMError::new(StatusCode::ABORTED)
            .with_sub_status(ERROR_OBJECT_ALREADY_BORROWED)
            .with_message(format!("Object {} already borrowed", object_id)));
    }

    gv.borrow_global()
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub as_ref_inner: AsRefGasParameters,
    pub as_mut_ref_inner: AsMutRefGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            as_ref_inner: AsRefGasParameters::zeros(),
            as_mut_ref_inner: AsMutRefGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "as_ref_inner",
            make_native_as_ref_inner(gas_params.as_ref_inner),
        ),
        (
            "as_mut_ref_inner",
            make_native_as_mut_ref_inner(gas_params.as_mut_ref_inner),
        ),
    ];

    make_module_natives(natives)
}

fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}
