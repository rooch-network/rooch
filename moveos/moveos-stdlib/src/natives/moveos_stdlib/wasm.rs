// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::values::Value;
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct WASMCreateInstanceGasParameters {
    pub base_create_instance: InternalGas,
    pub per_byte_instance: InternalGasPerByte,
}

impl WASMCreateInstanceGasParameters {
    pub fn zeros() -> Self {
        Self {
            base_create_instance: 0.into(),
            per_byte_instance: 0.into(),
        }
    }
}

#[inline]
fn native_create_wasm_instance(
    _gas_params: &WASMCreateInstanceGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    Ok(NativeResult::Success {
        cost: InternalGas::new(100),
        ret_vals: smallvec![Value::u128(88888)],
    })
}

#[derive(Debug, Clone)]
pub struct WASMCreateArgsGasParameters {
    pub base_create_args: InternalGas,
    pub per_byte_args: InternalGasPerByte,
}

impl WASMCreateArgsGasParameters {
    pub fn zeros() -> Self {
        Self {
            base_create_args: 0.into(),
            per_byte_args: 0.into(),
        }
    }
}

#[inline]
fn native_create_wasm_args(
    _gas_params: &WASMCreateArgsGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    Ok(NativeResult::Success {
        cost: InternalGas::new(100),
        ret_vals: smallvec![Value::u128(88888)],
    })
}

#[derive(Debug, Clone)]
pub struct WASMExecuteGasParameters {
    pub base_create_execution: InternalGas,
    pub per_byte_execution_result: InternalGasPerByte,
}

impl WASMExecuteGasParameters {
    pub fn zeros() -> Self {
        Self {
            base_create_execution: 0.into(),
            per_byte_execution_result: 0.into(),
        }
    }
}

#[inline]
fn native_execute_wasm_function(
    _gas_params: &WASMExecuteGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    Ok(NativeResult::Success {
        cost: InternalGas::new(100),
        ret_vals: smallvec![Value::u128(88888)],
    })
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub create_instance_gas_parameter: WASMCreateInstanceGasParameters,
    pub create_args_gas_parameter: WASMCreateArgsGasParameters,
    pub function_execution_gas_parameter: WASMExecuteGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            create_instance_gas_parameter: WASMCreateInstanceGasParameters::zeros(),
            create_args_gas_parameter: WASMCreateArgsGasParameters::zeros(),
            function_execution_gas_parameter: WASMExecuteGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "native_create_wasm_instance",
            make_native(
                gas_params.create_instance_gas_parameter,
                native_create_wasm_instance,
            ),
        ),
        (
            "native_create_wasm_args",
            make_native(
                gas_params.create_args_gas_parameter,
                native_create_wasm_args,
            ),
        ),
        (
            "native_execute_wasm_function",
            make_native(
                gas_params.function_execution_gas_parameter,
                native_execute_wasm_function,
            ),
        ),
    ];

    make_module_natives(natives)
}
