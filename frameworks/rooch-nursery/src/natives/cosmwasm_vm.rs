// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use log::{info, warn};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::{Struct, Value};
use serde_json::Value as JSONValue;
use smallvec::{smallvec, SmallVec};
use std::collections::VecDeque;
use std::ffi::CString;
use std::ops::Deref;
use std::vec;

use moveos_wasm::wasm::{
    create_wasm_instance, get_instance_pool, insert_wasm_instance, put_data_on_stack,
};

use moveos_stdlib::natives::helpers::{make_module_natives, make_native};

use crate::natives::helper::pop_object_id;

const E_WASM_ERROR: u64 = 1;

#[derive(Debug, Clone)]
pub struct CosmWasmCreateInstanceGasParameters {
    pub base: InternalGas,
}

impl CosmWasmCreateInstanceGasParameters {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

/***************************************************************************************************
 * native fun native_create_instance
 **************************************************************************************************/
#[inline]
fn native_create_instance(
    gas_parameters: &CosmWasmCreateInstanceGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert!(ty_args.len() == 0, "Wrong number of type arguments");
    assert!(args.len() == 2, "Wrong number of arguments");

    let store_obj_id = pop_object_id(&mut args)?;
    let wasm_bytes = pop_arg!(args, Vec<u8>);

    info!("native_create_instance wasm_bytes:{:?}", wasm_bytes);
    info!("native_create_instance store_obj_id:{:?}", store_obj_id);

    // Mock implementation: Always return a fixed instance ID
    let instance_id = b"mock_instance_id".to_vec();
    let result_code = 0u64; // 0 indicates success

    Ok(NativeResult::ok(
        gas_parameters.base,
        smallvec![Value::vector_u8(instance_id), Value::u64(result_code),],
    ))
}

#[derive(Debug, Clone)]
pub struct CosmWasmDestroyInstanceGasParameters {
    pub base: InternalGas,
}

impl CosmWasmDestroyInstanceGasParameters {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

/***************************************************************************************************
 * native fun native_destroy_instance
 **************************************************************************************************/
#[inline]
fn native_destroy_instance(
    gas_params: &CosmWasmDestroyInstanceGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert!(ty_args.len() == 2, "Wrong number of type arguments");
    assert!(arguments.len() == 1, "Wrong number of arguments");

    Ok(NativeResult::ok(gas_params.base, smallvec![Value::u64(0)]))
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub create_instance: CosmWasmCreateInstanceGasParameters,
    pub destroy_instance: CosmWasmDestroyInstanceGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            create_instance: CosmWasmCreateInstanceGasParameters::zeros(),
            destroy_instance: CosmWasmDestroyInstanceGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "native_create_instance",
            make_native(gas_params.create_instance, native_create_instance),
        ),
        (
            "native_destroy_instance",
            make_native(gas_params.destroy_instance, native_destroy_instance),
        ),
    ];

    make_module_natives(natives)
}
