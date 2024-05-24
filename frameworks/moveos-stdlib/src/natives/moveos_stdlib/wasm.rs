// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
use std::ffi::CString;
use std::ops::Deref;
use std::vec;

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::{Struct, Value};
use serde_json::Value as JSONValue;
use smallvec::smallvec;

use moveos_wasm::wasm::{
    create_wasm_instance, get_instance_pool, insert_wasm_instance, put_data_on_stack,
};

use crate::natives::helpers::{make_module_natives, make_native};

const E_INSTANCE_NO_EXISTS: u64 = 1;
// const E_ARG_NOT_U32: u64 = 2;
const E_ARG_NOT_VECTOR_U8: u64 = 3;
// const E_JSON_MARSHAL_FAILED: u64 = 4;
const E_WASM_EXECUTION_FAILED: u64 = 5;
const E_WASM_FUNCTION_NOT_FOUND: u64 = 6;
const E_WASM_MEMORY_ACCESS_FAILED: u64 = 7;
const E_JSON_UNMARSHAL_FAILED: u64 = 8;
pub const E_CBOR_MARSHAL_FAILED: u64 = 9;
pub const E_EMPTY_RETURN_VALUE: u64 = 10;
pub const E_VALUE_NOT_I32: u64 = 11;
pub const E_MEMORY_NOT_FOUND: u64 = 12;
pub const E_INCORRECT_LENGTH_OF_ARGS: u64 = 13;
pub const E_CBOR_UNMARSHAL_FAILED: u64 = 14;
pub const E_GET_INSTANCE_POOL_FAILED: u64 = 15;
pub const E_UNPACK_STRUCT_FAILED: u64 = 16;
pub const E_WASM_INSTANCE_CREATION_FAILED: u64 = 17;
pub const E_WASM_REMOVE_INSTANCE_FAILED: u64 = 18;

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

// native_create_wasm_instance
#[inline]
fn native_create_wasm_instance(
    gas_params: &WASMCreateInstanceGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 1 {
        return Ok(NativeResult::err(
            gas_params.base_create_instance,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let wasm_bytes = pop_arg!(args, Vec<u8>);
    let (instance_id, error_code) = create_wasm_instance(&wasm_bytes)
        .map(|instance| {
            match insert_wasm_instance(instance) {
                Ok(id) => (id, 0), // No error
                Err(_) => (0, E_WASM_INSTANCE_CREATION_FAILED),
            }
        })
        .unwrap_or((0, E_WASM_INSTANCE_CREATION_FAILED));

    let mut cost = gas_params.base_create_instance;
    cost += gas_params.per_byte_instance * NumBytes::new(wasm_bytes.len() as u64);

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::u64(instance_id), Value::u64(error_code)],
    ))
}

#[derive(Debug, Clone)]
pub struct WASMCreateCBORValue {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl WASMCreateCBORValue {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

// native_create_cbor_values
#[inline]
fn native_create_cbor_values(
    gas_params: &WASMCreateCBORValue,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 1 {
        return Ok(NativeResult::err(
            gas_params.base,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let value_list = pop_arg!(args, Vec<Value>);

    let mut func_args = Vec::new();
    for arg_value in value_list.iter() {
        let value = arg_value.copy_value()?;
        match value.value_as::<Vec<u8>>() {
            Ok(v) => func_args.push(String::from_utf8_lossy(v.as_slice()).to_string()),
            Err(_) => {
                return Ok(NativeResult::err(gas_params.base, E_ARG_NOT_VECTOR_U8));
            }
        }
    }

    let mut mint_args_json: Vec<JSONValue> = vec![];

    for arg in func_args.iter() {
        let arg_json_opt = serde_json::from_str(arg.as_str());
        let arg_json: JSONValue = match arg_json_opt {
            Ok(v) => v,
            Err(_) => {
                return Ok(NativeResult::err(gas_params.base, E_JSON_UNMARSHAL_FAILED));
            }
        };
        mint_args_json.push(arg_json);
    }

    let mint_args_array = JSONValue::Array(mint_args_json);
    log::debug!(
        "native_create_cbor_values -> mint_args_array {:?}",
        mint_args_array
    );
    let mut cbor_buffer = Vec::new();
    match ciborium::into_writer(&mint_args_array, &mut cbor_buffer) {
        Ok(_) => {}
        Err(_) => {
            return Ok(NativeResult::err(gas_params.base, E_CBOR_MARSHAL_FAILED));
        }
    }

    let cbor_value: ciborium::Value = match ciborium::from_reader(cbor_buffer.as_slice()) {
        Ok(v) => v,
        Err(_) => return build_err(gas_params.base, E_CBOR_UNMARSHAL_FAILED),
    };
    log::debug!(
        "native_create_cbor_values -> mint_args_array {:?}",
        cbor_value
    );

    let ret_vec = Value::vector_u8(cbor_buffer.clone());

    let mut cost = gas_params.base;
    cost += gas_params.per_byte * NumBytes::new(cbor_buffer.len() as u64);

    Ok(NativeResult::Success {
        cost,
        ret_vals: smallvec![ret_vec],
    })
}

#[derive(Debug, Clone)]
pub struct WASMCreateAddLength {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl WASMCreateAddLength {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

// native_add_length_with_data
#[inline]
fn native_add_length_with_data(
    gas_params: &WASMCreateAddLength,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 1 {
        return Ok(NativeResult::err(
            gas_params.base,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let mut data = pop_arg!(args, Vec<u8>);

    let mut buffer_final = Vec::new();
    buffer_final.append(&mut (data.len() as u32).to_be_bytes().to_vec());
    buffer_final.append(&mut data);

    let ret_vec = Value::vector_u8(buffer_final.clone());

    let mut cost = gas_params.base;
    cost += gas_params.per_byte * NumBytes::new(buffer_final.len() as u64);

    Ok(NativeResult::Success {
        cost,
        ret_vals: smallvec![ret_vec],
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

// native_create_wasm_args_in_memory
#[inline]
fn native_create_wasm_args_in_memory(
    gas_params: &WASMCreateArgsGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 3 {
        return Ok(NativeResult::err(
            gas_params.base_create_args,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let func_args_value = pop_arg!(args, Vec<Value>);
    let _func_name = pop_arg!(args, Vec<u8>); // TODO: check the length of function arguments
    let instance_id = pop_arg!(args, u64);
    let mut func_args = Vec::new();
    for arg_value in func_args_value.iter() {
        let value = arg_value.copy_value()?;
        match value.value_as::<Vec<u8>>() {
            Ok(v) => func_args.push(v),
            Err(_) => {
                return Ok(NativeResult::err(
                    gas_params.base_create_args,
                    E_ARG_NOT_VECTOR_U8,
                ));
            }
        }
    }

    let mut args_bytes_total = 0;

    let mut data_ptr_list = Vec::new();

    let instance_pool = get_instance_pool();
    let mut pool_object = match instance_pool.lock() {
        Ok(v) => v,
        Err(_) => {
            return Ok(NativeResult::err(
                gas_params.base_create_args,
                E_GET_INSTANCE_POOL_FAILED,
            ))
        }
    };
    match pool_object.get_mut(&instance_id) {
        None => {
            return Ok(NativeResult::err(
                gas_params.base_create_args,
                E_INSTANCE_NO_EXISTS,
            ));
        }
        Some(instance) => {
            for arg in func_args.iter() {
                let c_arg = unsafe { CString::from_vec_unchecked(arg.clone()) };

                let mut arg_buffer = Vec::new();
                // arg_buffer.append(&mut (arg.len() as u32).to_be_bytes().to_vec());
                arg_buffer.append(&mut c_arg.into_bytes_with_nul());
                let buffer_final_ptr = match put_data_on_stack(instance, arg_buffer.as_slice()) {
                    Ok(v) => v,
                    Err(_) => {
                        return build_err(gas_params.base_create_args, E_WASM_EXECUTION_FAILED)
                    }
                };

                data_ptr_list.push(buffer_final_ptr as u64);
                args_bytes_total += arg.len();
            }
        }
    }

    let mut cost = gas_params.base_create_args;
    cost += gas_params.per_byte_args * NumBytes::new(args_bytes_total as u64);

    let return_value_list = Value::vector_u64(data_ptr_list);
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

fn build_err(cost: InternalGas, abort_code: u64) -> PartialVMResult<NativeResult> {
    Ok(NativeResult::err(cost, abort_code))
}

// native_execute_wasm_function
#[inline]
fn native_execute_wasm_function(
    gas_params: &WASMExecuteGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 3 {
        return Ok(NativeResult::err(
            gas_params.base_create_execution,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let func_args = pop_arg!(args, Vec<u64>);
    let func_name = pop_arg!(args, Vec<u8>);
    let instance_id = pop_arg!(args, u64);

    let instance_pool = get_instance_pool();
    let mut pool_object = match instance_pool.lock() {
        Ok(v) => v,
        Err(_) => {
            return Ok(NativeResult::err(
                gas_params.base_create_execution,
                E_GET_INSTANCE_POOL_FAILED,
            ))
        }
    };
    let ret = match pool_object.get_mut(&instance_id) {
        None => Ok(NativeResult::err(
            gas_params.base_create_execution,
            E_INSTANCE_NO_EXISTS,
        )),
        Some(instance) => {
            match instance.instance.exports.get_function(
                String::from_utf8_lossy(func_name.as_slice())
                    .to_string()
                    .as_str(),
            ) {
                Ok(calling_function) => {
                    let mut wasm_func_args = Vec::new();
                    for arg in func_args.iter() {
                        wasm_func_args.push(wasmer::Value::I32(*arg as i32));
                    }

                    // TODO: check the length of arguments for the function calling

                    match calling_function.call(&mut instance.store, wasm_func_args.as_slice()) {
                        Ok(ret) => {
                            let return_value = match ret.deref().first() {
                                Some(v) => v,
                                None => {
                                    return build_err(
                                        gas_params.base_create_execution,
                                        E_EMPTY_RETURN_VALUE,
                                    )
                                }
                            };
                            let offset = match return_value.i32() {
                                Some(v) => v,
                                None => {
                                    return build_err(
                                        gas_params.base_create_execution,
                                        E_VALUE_NOT_I32,
                                    )
                                }
                            };
                            let ret_val = Value::u64(offset as u64);

                            let mut cost = gas_params.base_create_execution;
                            cost += gas_params.per_byte_execution_result
                                * NumBytes::new(instance.bytecode.len() as u64);

                            Ok(NativeResult::Success {
                                cost,
                                ret_vals: smallvec![ret_val],
                            })
                        }
                        Err(_) => Ok(NativeResult::err(
                            gas_params.base_create_execution,
                            E_WASM_EXECUTION_FAILED,
                        )),
                    }
                }
                Err(_) => Ok(NativeResult::err(
                    gas_params.base_create_execution,
                    E_WASM_FUNCTION_NOT_FOUND,
                )),
            }
        }
    };
    ret
}

#[derive(Debug, Clone)]
pub struct WASMReadAddLength {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl WASMReadAddLength {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

// native_read_data_length
#[inline]
fn native_read_data_length(
    gas_params: &WASMReadAddLength,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 2 {
        return Ok(NativeResult::err(
            gas_params.base,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let data_ptr = pop_arg!(args, u64);
    let instance_id = pop_arg!(args, u64);

    let instance_pool = get_instance_pool();
    let mut pool_object = match instance_pool.lock() {
        Ok(v) => v,
        Err(_) => {
            return Ok(NativeResult::err(
                gas_params.base,
                E_GET_INSTANCE_POOL_FAILED,
            ))
        }
    };
    let ret = match pool_object.get_mut(&instance_id) {
        None => Ok(NativeResult::err(gas_params.base, E_INSTANCE_NO_EXISTS)),
        Some(instance) => {
            let memory = match instance.instance.exports.get_memory("memory") {
                Ok(v) => v,
                Err(_) => return build_err(gas_params.base, E_MEMORY_NOT_FOUND),
            };
            let memory_view = memory.view(&instance.store);
            let mut length_bytes: [u8; 4] = [0; 4];
            match memory_view.read(data_ptr, length_bytes.as_mut_slice()) {
                Ok(_) => {
                    let data_length = u32::from_be_bytes(length_bytes);
                    let cost = gas_params.base;

                    Ok(NativeResult::Success {
                        cost,
                        ret_vals: smallvec![Value::u32(data_length)],
                    })
                }
                Err(_) => Ok(NativeResult::err(
                    gas_params.base,
                    E_WASM_MEMORY_ACCESS_FAILED,
                )),
            }
        }
    };
    ret
}

#[derive(Debug, Clone)]
pub struct WASMReadHeapData {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl WASMReadHeapData {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

// native_read_data_from_heap
#[inline]
fn native_read_data_from_heap(
    gas_params: &WASMReadHeapData,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    if args.len() != 3 {
        return Ok(NativeResult::err(
            gas_params.base,
            E_INCORRECT_LENGTH_OF_ARGS,
        ));
    }

    let data_length = pop_arg!(args, u32);
    let data_ptr = pop_arg!(args, u32);
    let instance_id = pop_arg!(args, u64);

    let instance_pool = get_instance_pool();
    let mut pool_object = match instance_pool.lock() {
        Ok(v) => v,
        Err(_) => {
            return Ok(NativeResult::err(
                gas_params.base,
                E_GET_INSTANCE_POOL_FAILED,
            ))
        }
    };
    let ret = match pool_object.get_mut(&instance_id) {
        None => Ok(NativeResult::err(gas_params.base, E_INSTANCE_NO_EXISTS)),
        Some(instance) => {
            let memory = match instance.instance.exports.get_memory("memory") {
                Ok(v) => v,
                Err(_) => return build_err(gas_params.base, E_MEMORY_NOT_FOUND),
            };
            let memory_view = memory.view(&instance.store);
            if data_length > 0 {
                let mut data = vec![0; data_length as usize];
                match memory_view.read(data_ptr as u64, &mut data) {
                    Ok(_) => {
                        let mut cost = gas_params.base;
                        cost += gas_params.per_byte * NumBytes::new(data_length as u64);
                        Ok(NativeResult::Success {
                            cost,
                            ret_vals: smallvec![Value::vector_u8(data)],
                        })
                    }
                    Err(_) => Ok(NativeResult::err(
                        gas_params.base,
                        E_WASM_MEMORY_ACCESS_FAILED,
                    )),
                }
            } else {
                let mut c_char_array = Vec::new();
                let mut start_offset = data_ptr as u64;
                let mut bytes_read = 0;
                loop {
                    match memory_view.read_u8(start_offset) {
                        Ok(v) => {
                            if v == b'\0' {
                                let mut cost = gas_params.base;
                                cost += gas_params.per_byte * NumBytes::new(bytes_read);

                                return Ok(NativeResult::Success {
                                    cost,
                                    ret_vals: smallvec![Value::vector_u8(c_char_array)],
                                });
                            }
                            c_char_array.push(v);
                            start_offset += 1;
                            bytes_read += 1;
                        }
                        Err(_) => {
                            return Ok(NativeResult::err(
                                gas_params.base,
                                E_WASM_MEMORY_ACCESS_FAILED,
                            ));
                        }
                    }
                }
            }
        }
    };
    ret
}

#[derive(Debug, Clone)]
pub struct WASMReleaseInstance {
    pub base: InternalGas,
}

impl WASMReleaseInstance {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

// native_release_wasm_instance
#[inline]
fn native_release_wasm_instance(
    gas_params: &WASMReleaseInstance,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let value = match args.pop_back() {
        Some(v) => v,
        None => return build_err(gas_params.base, E_INCORRECT_LENGTH_OF_ARGS),
    };
    let mut fields = match value.value_as::<Struct>() {
        Ok(struct_) => match struct_.unpack() {
            Ok(fields_iterator) => fields_iterator,
            Err(_) => return Ok(NativeResult::err(gas_params.base, E_UNPACK_STRUCT_FAILED)),
        },
        Err(_) => return Ok(NativeResult::err(gas_params.base, E_UNPACK_STRUCT_FAILED)),
    };
    let val = fields.next().ok_or_else(|| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE)
            .with_message("There must have only one field".to_owned())
    })?;

    let instance_id = val.value_as::<u64>()?;

    {
        let instance_pool = get_instance_pool();
        let mut pool_object = match instance_pool.lock() {
            Ok(v) => v,
            Err(_) => {
                return Ok(NativeResult::err(
                    gas_params.base,
                    E_GET_INSTANCE_POOL_FAILED,
                ))
            }
        };
        match pool_object.get_mut(&instance_id) {
            None => return Ok(NativeResult::err(gas_params.base, E_INSTANCE_NO_EXISTS)),
            Some(_) => {}
        };
    }

    match moveos_wasm::wasm::remove_instance(instance_id) {
        Ok(_) => {}
        Err(_) => {
            return Ok(NativeResult::err(
                gas_params.base,
                E_GET_INSTANCE_POOL_FAILED,
            ))
        }
    };

    Ok(NativeResult::Success {
        cost: gas_params.base,
        ret_vals: smallvec![Value::bool(true)],
    })
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub create_instance_gas_parameter: WASMCreateInstanceGasParameters,
    pub create_cbor_value_gas_parameter: WASMCreateCBORValue,
    pub add_length_with_data: WASMCreateAddLength,
    pub create_args_gas_parameter: WASMCreateArgsGasParameters,
    pub function_execution_gas_parameter: WASMExecuteGasParameters,
    pub read_data_length_gas_parameter: WASMReadAddLength,
    pub read_heap_data: WASMReadHeapData,
    pub release_wasm_instance: WASMReleaseInstance,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            create_instance_gas_parameter: WASMCreateInstanceGasParameters::zeros(),
            create_cbor_value_gas_parameter: WASMCreateCBORValue::zeros(),
            add_length_with_data: WASMCreateAddLength::zeros(),
            create_args_gas_parameter: WASMCreateArgsGasParameters::zeros(),
            function_execution_gas_parameter: WASMExecuteGasParameters::zeros(),
            read_data_length_gas_parameter: WASMReadAddLength::zeros(),
            read_heap_data: WASMReadHeapData::zeros(),
            release_wasm_instance: WASMReleaseInstance::zeros(),
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
            "native_create_cbor_values",
            make_native(
                gas_params.create_cbor_value_gas_parameter,
                native_create_cbor_values,
            ),
        ),
        (
            "native_add_length_with_data",
            make_native(gas_params.add_length_with_data, native_add_length_with_data),
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
        (
            "native_read_data_length",
            make_native(
                gas_params.read_data_length_gas_parameter,
                native_read_data_length,
            ),
        ),
        (
            "native_read_data_from_heap",
            make_native(gas_params.read_heap_data, native_read_data_from_heap),
        ),
        (
            "native_release_wasm_instance",
            make_native(
                gas_params.release_wasm_instance,
                native_release_wasm_instance,
            ),
        ),
    ];

    make_module_natives(natives)
}
