// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use log::{debug, error, warn};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ffi::CString;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::vec;

use cosmwasm_std::{Checksum, Empty};
use cosmwasm_vm::{
    call_instantiate, capabilities_from_csv, Cache, CacheOptions, Instance, InstanceOptions, Size,
};
use once_cell::sync::Lazy;
use rooch_cosmwasm_vm::backend::{
    build_mock_backend, MoveBackendApi, MoveBackendQuerier, MoveStorage,
};
use serde_json::Value as JSONValue;
use smallvec::{smallvec, SmallVec};

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::{Struct, Value};

use moveos_object_runtime::{
    runtime::ObjectRuntimeContext, runtime_object::RuntimeObject, TypeLayoutLoader,
};
use moveos_types::{
    moveos_std::object::ObjectID, state::FieldKey, state_resolver::StatelessResolver,
};

use moveos_stdlib::natives::helpers::{make_module_natives, make_native};

use crate::natives::helper::{pop_object_id, CommonGasParameters};

const DEFAULT_GAS_LIMIT: u64 = 10000;
const E_WASM_ERROR: u64 = 1;

static WASM_CACHE: Lazy<Arc<Cache<MoveBackendApi, MoveStorage, MoveBackendQuerier>>> =
    Lazy::new(|| {
        let options = CacheOptions::new(
            std::env::temp_dir(),
            capabilities_from_csv("iterator,staking"),
            Size::mebi(200),
            Size::mebi(64),
        );
        Arc::new(unsafe { Cache::new(options).unwrap() })
    });

#[derive(Debug, Clone)]
pub struct CosmWasmCreateInstanceGasParameters {
    pub base: InternalGas,
    pub per_byte_wasm: InternalGasPerByte,
}

impl CosmWasmCreateInstanceGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte_wasm: InternalGasPerByte::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun native_create_instance
 **************************************************************************************************/
#[inline]
pub fn native_create_instance(
    gas_parameters: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(
        ty_args.is_empty(),
        "native_create_instance expects no type arguments"
    );
    debug_assert_eq!(args.len(), 2, "native_create_instance expects 2 arguments");

    let common_gas_parameter = gas_parameters.common.clone();
    let create_instance_gas_parameter = gas_parameters.native_create_instance.clone();

    let store_obj_id = pop_object_id(&mut args)?;
    let wasm_code = pop_arg!(args, Vec<u8>);

    wasm_instance_fn_dispatch(
        &common_gas_parameter,
        create_instance_gas_parameter.base,
        create_instance_gas_parameter.per_byte_wasm,
        context,
        store_obj_id,
        wasm_code,
        move |_layout_loader,
              _resolver,
              _rt_obj,
              wasm_bytes|
              -> PartialVMResult<(Value, Option<Option<NumBytes>>)> {
            // wat2 wasm bytes
            let bytecode = wasmer::wat2wasm(wasm_bytes.as_slice()).map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(format!("Failed to cast wat to WASM: {}", e))
            })?;

            // Save WASM bytecode and get checksum
            let checksum = WASM_CACHE.save_wasm(&bytecode).map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(format!("Failed to save WASM: {}", e))
            })?;

            //TODO Create real Backend
            let backend = build_mock_backend();

            // Create WASM instance
            let instance_options = InstanceOptions {
                gas_limit: DEFAULT_GAS_LIMIT,
            };

            let (module, store) = WASM_CACHE.get_module(&checksum).map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(format!("Failed to get WASM module: {}", e))
            })?;

            let _ = Instance::from_module(
                store,
                &module,
                backend,
                instance_options.gas_limit,
                None,
                None,
            )
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(format!("Failed to get WASM instance: {}", e))
            })?;

            Ok((
                Value::vector_u8(checksum.as_slice().to_vec()),
                Some(Some(NumBytes::new(wasm_bytes.len() as u64))),
            ))
        },
    )
}

fn wasm_instance_fn_dispatch(
    common_gas_params: &CommonGasParameters,
    base: InternalGas,
    per_byte_wasm: InternalGasPerByte,
    context: &mut NativeContext,
    store_obj_id: ObjectID,
    wasm_bytes: Vec<u8>,
    f: impl FnOnce(
        &dyn TypeLayoutLoader,
        &dyn StatelessResolver,
        &mut RuntimeObject,
        Vec<u8>,
    ) -> PartialVMResult<(Value, Option<Option<NumBytes>>)>,
) -> PartialVMResult<NativeResult> {
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();
    let resolver = object_runtime.resolver();
    let (rt_obj, object_load_gas) = object_runtime.load_object(context, &store_obj_id)?;
    let wasm_bytes_len = wasm_bytes.len() as u64;
    let gas_cost = base
        + per_byte_wasm * NumBytes::new(wasm_bytes_len)
        + common_gas_params.calculate_load_cost(object_load_gas);

    let result = f(context, resolver, rt_obj, wasm_bytes);
    match result {
        Ok((value, wasm_load_gas)) => Ok(NativeResult::ok(
            gas_cost + common_gas_params.calculate_load_cost(wasm_load_gas),
            smallvec![value, Value::u32(0)],
        )),
        Err(err) => {
            error!("wasm_instance_fn_dispatch error: {:?}", err);
            let abort_code = error_to_abort_code(err);
            Ok(NativeResult::ok(
                gas_cost,
                smallvec![Value::vector_u8(vec![]), Value::u32(abort_code as u32)],
            ))
        }
    }
}

// Helper function: convert PartialVMError to abort code
fn error_to_abort_code(err: PartialVMError) -> u64 {
    match err.major_status() {
        _ => err.major_status().into(),
    }
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
    gas_params: &GasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert!(ty_args.len() == 0, "Wrong number of type arguments");
    assert!(arguments.len() == 1, "Wrong number of arguments");

    Ok(NativeResult::ok(
        gas_params.common.load_base,
        smallvec![Value::u32(0)],
    ))
}

#[inline]
fn native_call_instantiate_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(
        ty_args.is_empty(),
        "native_call_instantiate_raw expects no type arguments"
    );
    debug_assert!(
        arguments.len() == 5,
        "native_call_instantiate_raw expects 5 arguments"
    );

    let code_checksum = pop_arg!(arguments, Vec<u8>);
    let store_obj_id = pop_object_id(&mut arguments)?;
    let env = pop_arg!(arguments, Vec<u8>);
    let info = pop_arg!(arguments, Vec<u8>);
    let msg = pop_arg!(arguments, Vec<u8>);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();
    let resolver = object_runtime.resolver();
    let (rt_obj, object_load_gas) = object_runtime.load_object(context, &store_obj_id)?;

    let gas_cost = gas_params.common.load_base
        + gas_params
            .common
            .calculate_load_cost(Some(Some(NumBytes::new(code_checksum.len() as u64))));

    let result = instantiate_contract(context, resolver, rt_obj, code_checksum, env, info, msg);

    match result {
        Ok((response, wasm_gas_used)) => {
            let total_gas = gas_cost + InternalGas::new(wasm_gas_used);
            Ok(NativeResult::ok(
                total_gas,
                smallvec![
                    Value::vector_u8(response),
                    Value::u32(0) // success
                ],
            ))
        }
        Err(err) => {
            let error_code = error_to_abort_code(err);
            Ok(NativeResult::ok(
                gas_cost,
                smallvec![Value::vector_u8(vec![]), Value::u32(error_code as u32)],
            ))
        }
    }
}

fn instantiate_contract(
    context: &NativeContext,
    resolver: &dyn StatelessResolver,
    rt_obj: &mut RuntimeObject,
    code_checksum: Vec<u8>,
    env: Vec<u8>,
    info: Vec<u8>,
    msg: Vec<u8>,
) -> PartialVMResult<(Vec<u8>, u64)> {
    let checksum = Checksum::try_from(code_checksum.as_slice()).map_err(|e| vm_error(e))?;
    let (module, store) = WASM_CACHE.get_module(&checksum).map_err(|e| vm_error(e))?;

    let backend = build_mock_backend();
    let instance_options = InstanceOptions {
        gas_limit: DEFAULT_GAS_LIMIT,
    };
    let mut instance = Instance::from_module(
        store,
        &module,
        backend,
        instance_options.gas_limit,
        None,
        None,
    )
    .map_err(|e| {
        PartialVMError::new(StatusCode::STORAGE_ERROR)
            .with_message(format!("Failed to get WASM instance: {}", e))
    })?;

    let env = serde_json::from_slice::<cosmwasm_std::Env>(&env).map_err(|e| vm_error(e))?;
    let info =
        serde_json::from_slice::<cosmwasm_std::MessageInfo>(&info).map_err(|e| vm_error(e))?;

    let result = call_instantiate::<_, _, _, Empty>(&mut instance, &env, &info, msg.as_slice())
        .map_err(|e| vm_error(e))?;

    let response = serde_json::to_vec(&result).map_err(|e| vm_error(e))?;
    let gas_used = instance.get_gas_left();

    Ok((response, gas_used))
}

fn vm_error(err: impl std::fmt::Display) -> PartialVMError {
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(format!("{}", err))
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: CommonGasParameters,
    pub native_create_instance: CosmWasmCreateInstanceGasParameters,
    pub native_destroy_instance: CosmWasmDestroyInstanceGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParameters::zeros(),
            native_create_instance: CosmWasmCreateInstanceGasParameters::zeros(),
            native_destroy_instance: CosmWasmDestroyInstanceGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "native_create_instance",
            make_native(gas_params.clone(), native_create_instance),
        ),
        (
            "native_destroy_instance",
            make_native(gas_params.clone(), native_destroy_instance),
        ),
    ];

    make_module_natives(natives)
}
