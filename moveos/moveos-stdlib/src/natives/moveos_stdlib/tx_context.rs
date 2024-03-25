// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::{
    helpers::make_module_natives, moveos_stdlib::raw_table::ObjectRuntimeContext,
};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

#[derive(Debug, Clone)]
pub struct BorrowGasParameters {
    pub base: InternalGas,
}

impl BorrowGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun borrow
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_borrow(
    gas_params: &BorrowGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.is_empty());

    let tx_context = borrow_tx_context(context)?;
    Ok(NativeResult::ok(gas_params.base, smallvec![tx_context]))
}

pub fn make_native_borrow(gas_params: BorrowGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| native_borrow(&gas_params, context, ty_args, args))
}

#[derive(Debug, Clone)]
pub struct BorrowMutGasParameters {
    pub base: InternalGas,
}

impl BorrowMutGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun borrow_mut
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_borrow_mut(
    gas_params: &BorrowMutGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.is_empty());

    let tx_context = borrow_tx_context(context)?;
    Ok(NativeResult::ok(gas_params.base, smallvec![tx_context]))
}

pub fn make_native_borrow_mut(gas_params: BorrowMutGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| native_borrow_mut(&gas_params, context, ty_args, args))
}

fn borrow_tx_context(context: &mut NativeContext) -> PartialVMResult<Value> {
    let object_context = context.extensions_mut().get_mut::<ObjectRuntimeContext>();

    let data = object_context.object_runtime();
    let object_runtime = data.read();
    object_runtime.tx_context.borrow_global()
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub borrow_inner: BorrowGasParameters,
    pub borrow_mut_inner: BorrowMutGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            borrow_inner: BorrowGasParameters::zeros(),
            borrow_mut_inner: BorrowMutGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("borrow_inner", make_native_borrow(gas_params.borrow_inner)),
        (
            "borrow_mut_inner",
            make_native_borrow_mut(gas_params.borrow_mut_inner),
        ),
    ];

    make_module_natives(natives)
}
