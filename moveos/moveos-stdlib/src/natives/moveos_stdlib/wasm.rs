// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use smallvec::{smallvec, SmallVec};
use wasmer::{AsStoreMut, Store};

use moveos_wasm::wasm::{
    create_wasm_instance, get_instance_pool, insert_wasm_instance, put_data_on_stack, WASMInstance,
    GLOBAL_MEMORY,
};

use crate::natives::helpers::{make_module_natives, make_native};

const E_INSTANCE_NO_EXISTS: u64 = 1;

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
    gas_params: &WASMCreateInstanceGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let wasm_bytes = pop_arg!(args, Vec<u8>);
    let wasm_instance = create_wasm_instance(&wasm_bytes);
    let instance_id = insert_wasm_instance(wasm_instance);

    let mut cost = gas_params.base_create_instance;
    cost += gas_params.per_byte_instance * NumBytes::new(wasm_bytes.len() as u64);

    Ok(NativeResult::Success {
        cost,
        ret_vals: smallvec![Value::u64(instance_id)],
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
fn native_create_wasm_args_in_memory(
    gas_params: &WASMCreateArgsGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let func_args_value = pop_arg!(args, Vec<Value>);
    let func_name = pop_arg!(args, Vec<u8>);
    let instance_id = pop_arg!(args, u64);
    let mut func_args = Vec::new();
    for arg_value in func_args_value.iter() {
        let value = arg_value.copy_value()?;
        match value.value_as::<Vec<u8>>() {
            Ok(v) => func_args.push(v),
            Err(_) => {}
        }
    }

    let mut args_bytes_total = 0;

    let mut data_ptr_list = Vec::new();

    let instance_pool = get_instance_pool();
    match instance_pool.lock().unwrap().get_mut(&instance_id) {
        None => {
            return Ok(NativeResult::err(
                gas_params.base_create_args,
                E_INSTANCE_NO_EXISTS,
            ));
        }
        Some(instance) => {
            let stack_alloc_func = instance
                .instance
                .exports
                .get_function("stackAlloc")
                .unwrap();

            for arg in func_args.iter() {
                let mut arg_buffer = Vec::new();
                arg_buffer.append(&mut (arg.len() as u32).to_be_bytes().to_vec());
                arg_buffer.append(&mut arg.clone());
                let buffer_final_ptr =
                    put_data_on_stack(stack_alloc_func, &mut instance.store, arg_buffer.as_slice());

                data_ptr_list.push(buffer_final_ptr as u32);
                args_bytes_total += arg.len();
            }
        }
    }

    let mut cost = gas_params.base_create_args;
    cost += gas_params.per_byte_args * NumBytes::new(args_bytes_total as u64);

    println!(
        "111111111 {:?} {:?} {:?} -> {:?}",
        instance_id,
        String::from_utf8_lossy(func_name.as_slice()),
        func_args,
        data_ptr_list
    );

    let mut return_value_list = Value::vector_u32(data_ptr_list);
    Ok(NativeResult::Success {
        cost,
        ret_vals: smallvec![return_value_list],
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
    let wasm_bytes = pop_arg!(_args, Vec<u8>);
    let ret_val = Value::u64(88888u64);
    Ok(NativeResult::Success {
        cost: InternalGas::new(100),
        ret_vals: smallvec![ret_val],
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
            "native_create_wasm_args_in_memory",
            make_native(
                gas_params.create_args_gas_parameter,
                native_create_wasm_args_in_memory,
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
