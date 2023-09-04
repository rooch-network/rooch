// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use std::{collections::VecDeque, sync::Arc};

use crate::natives::helpers::make_module_natives;

#[derive(Debug, Clone)]
pub struct ToBytesGasParameters {
    pub base: InternalGas,
}

impl ToBytesGasParameters {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

/// Rust implementation of Move's `native public fun to_bytes<T>(&T): vector<u8> in rlp module`
#[inline]
fn native_to_bytes(
    _gas_params: &ToBytesGasParameters,
    _context: &mut NativeContext,
    mut _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    todo!()
}
pub fn make_native_to_bytes(gas_params: ToBytesGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_to_bytes(&gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

/// Rust implementation of Move's `native public(friend) fun from_bytes<T>(vector<u8>): T in rlp module`
#[inline]
fn native_from_bytes(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    mut _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    todo!()
}

pub fn make_native_from_bytes(gas_params: FromBytesGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_from_bytes(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub to_bytes: ToBytesGasParameters,
    pub from_bytes: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            to_bytes: ToBytesGasParameters::zeros(),
            from_bytes: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("to_bytes", make_native_to_bytes(gas_params.to_bytes)),
        ("from_bytes", make_native_from_bytes(gas_params.from_bytes)),
    ];

    make_module_natives(natives)
}
