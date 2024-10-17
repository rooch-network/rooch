// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use log::error;
use std::collections::VecDeque;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::vec;

use cosmwasm_std::Checksum;
use cosmwasm_vm::{
    call_execute_raw, call_instantiate_raw, call_migrate_raw, call_query_raw, call_reply_raw,
    call_sudo_raw, capabilities_from_csv, Cache, CacheOptions, Instance, InstanceOptions, Size,
    VmResult, Storage,
};
use rooch_cosmwasm_vm::{
    build_move_proxy_backend, ProxyStorage, MoveStorage, MoveBackendApi, MoveBackendQuerier,
};

use once_cell::sync::Lazy;
use smallvec::smallvec;

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::Value;

use moveos_object_runtime::{
    runtime::ObjectRuntimeContext, runtime_object::RuntimeObject, TypeLayoutLoader,
};
use moveos_types::{moveos_std::object::ObjectID, state_resolver::StatelessResolver};

use moveos_stdlib::natives::helpers::{make_module_natives, make_native};

use crate::natives::helper::{pop_object_id, CommonGasParametersOption};

const DEFAULT_GAS_LIMIT: u64 = 10000000;

static WASM_CACHE: Lazy<Arc<Cache<MoveBackendApi, ProxyStorage, MoveBackendQuerier>>> =
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
pub struct CosmWasmCreateInstanceGasParametersOption {
    pub base: Option<InternalGas>,
    pub per_byte_wasm: Option<InternalGasPerByte>,
}

impl CosmWasmCreateInstanceGasParametersOption {
    pub fn zeros() -> Self {
        Self {
            base: Some(0.into()),
            per_byte_wasm: Some(InternalGasPerByte::zero()),
        }
    }
}

fn vm_error(err: impl std::fmt::Display) -> PartialVMError {
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(format!("{}", err))
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
        create_instance_gas_parameter
            .base
            .unwrap_or_else(InternalGas::zero),
        create_instance_gas_parameter
            .per_byte_wasm
            .unwrap_or_else(InternalGasPerByte::zero),
        context,
        store_obj_id,
        wasm_code,
        move |layout_loader,
              resolver,
              rt_obj,
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
            let mut backend = build_move_proxy_backend();

            let move_storage = MoveStorage::new(rt_obj, layout_loader, resolver);
            let storage = Rc::new(RefCell::new(Box::new(move_storage) as Box<dyn Storage>));
            backend.storage.register(rt_obj.id().clone(), storage);

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
    common_gas_params: &CommonGasParametersOption,
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
    err.major_status().into()
}

#[derive(Debug, Clone)]
pub struct CosmWasmDestroyInstanceGasParametersOption {
    pub base: Option<InternalGas>,
}

impl CosmWasmDestroyInstanceGasParametersOption {
    pub fn zeros() -> Self {
        Self {
            base: Some(0.into()),
        }
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
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert!(ty_args.is_empty(), "Wrong number of type arguments");
    assert!(arguments.len() == 1, "Wrong number of arguments");

    Ok(NativeResult::ok(
        gas_params
            .common
            .load_base
            .unwrap_or_else(InternalGas::zero),
        smallvec![Value::u32(0)],
    ))
}

/***************************************************************************************************
 * native_call_instantiate_raw
 **************************************************************************************************/

fn native_contract_call<F>(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
    expected_args: usize,
    operation_name: &str,
    contract_operation: F,
) -> PartialVMResult<NativeResult>
where
    F: FnOnce(
        &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
        &[u8],
        Option<&[u8]>,
        &[u8],
    ) -> VmResult<Vec<u8>>,
{
    debug_assert!(
        ty_args.is_empty(),
        "{} expects no type arguments",
        operation_name
    );
    debug_assert_eq!(
        arguments.len(),
        expected_args,
        "{} expects {} arguments",
        operation_name,
        expected_args
    );

    let msg = pop_arg!(arguments, Vec<u8>);
    let info = if expected_args == 5 {
        Some(pop_arg!(arguments, Vec<u8>))
    } else {
        None
    };
    let env = pop_arg!(arguments, Vec<u8>);
    let store_obj_id = pop_object_id(&mut arguments)?;
    let code_checksum = pop_arg!(arguments, Vec<u8>);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();
    let _resolver = object_runtime.resolver();
    let (_rt_obj, object_load_gas) = object_runtime.load_object(context, &store_obj_id)?;

    let gas_cost = gas_params
        .common
        .load_base
        .unwrap_or_else(InternalGas::zero)
        + gas_params.common.calculate_load_cost(object_load_gas)
        + gas_params
            .common
            .calculate_load_cost(Some(Some(NumBytes::new(code_checksum.len() as u64))));

    let checksum = Checksum::try_from(code_checksum.as_slice()).map_err(vm_error)?;
    let (module, store) = WASM_CACHE.get_module(&checksum).map_err(vm_error)?;

    let backend = build_move_proxy_backend();
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
    .map_err(|e| vm_error(format!("Failed to get WASM instance: {}", e)))?;

    let result = contract_operation(
        &mut instance,
        env.as_slice(),
        info.as_ref().map(AsRef::as_ref),
        msg.as_slice(),
    );

    match result {
        Ok(response) => {
            let gas_used = instance.get_gas_left();
            let total_gas = gas_cost + InternalGas::new(gas_used);
            Ok(NativeResult::ok(
                total_gas,
                smallvec![
                    Value::vector_u8(response),
                    Value::u32(0) // success
                ],
            ))
        }
        Err(err) => {
            error!("{} error: {:?}", operation_name, err);

            let error_code = StatusCode::VM_EXTENSION_ERROR;
            Ok(NativeResult::ok(
                gas_cost,
                smallvec![Value::vector_u8(vec![]), Value::u32(error_code as u32)],
            ))
        }
    }
}

/***************************************************************************************************
 * call_instantiate_raw
 **************************************************************************************************/

#[inline]
fn native_call_instantiate_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    native_contract_call(
        gas_params,
        context,
        ty_args,
        arguments,
        5, // code_checksum, store_obj_id, env, info, msg
        "call_instantiate_raw",
        move |instance: &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
              env: &[u8],
              info: Option<&[u8]>,
              msg: &[u8]|
              -> VmResult<Vec<u8>> {
            call_instantiate_raw(instance, env, info.unwrap(), msg)
        },
    )
}

/***************************************************************************************************
 * native_call_execute_raw
 **************************************************************************************************/

#[inline]
fn native_call_execute_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    native_contract_call(
        gas_params,
        context,
        ty_args,
        arguments,
        5, // code_checksum, store_obj_id, env, info, msg
        "call_execute_raw",
        move |instance: &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
              env: &[u8],
              info: Option<&[u8]>,
              msg: &[u8]|
              -> VmResult<Vec<u8>> { call_execute_raw(instance, env, info.unwrap(), msg) },
    )
}

/***************************************************************************************************
 * native_call_query_raw
 **************************************************************************************************/

#[inline]
fn native_call_query_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    native_contract_call(
        gas_params,
        context,
        ty_args,
        arguments,
        4, // code_checksum, store_obj_id, env, msg
        "call_query_raw",
        move |instance: &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
              env: &[u8],
              _info: Option<&[u8]>,
              msg: &[u8]|
              -> VmResult<Vec<u8>> { call_query_raw(instance, env, msg) },
    )
}

/***************************************************************************************************
 * native_call_migrate_raw
 **************************************************************************************************/

#[inline]
fn native_call_migrate_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    native_contract_call(
        gas_params,
        context,
        ty_args,
        arguments,
        4, // code_checksum, store_obj_id, env, msg
        "call_migrate_raw",
        move |instance: &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
              env: &[u8],
              _info: Option<&[u8]>,
              msg: &[u8]|
              -> VmResult<Vec<u8>> { call_migrate_raw(instance, env, msg) },
    )
}

/***************************************************************************************************
 * native_call_reply_raw
 **************************************************************************************************/

#[inline]
fn native_call_reply_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    native_contract_call(
        gas_params,
        context,
        ty_args,
        arguments,
        4, // code_checksum, store_obj_id, env, msg
        "call_reply_raw",
        move |instance: &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
              env: &[u8],
              _info: Option<&[u8]>,
              msg: &[u8]|
              -> VmResult<Vec<u8>> { call_reply_raw(instance, env, msg) },
    )
}

/***************************************************************************************************
 * native_call_sudo_raw
 **************************************************************************************************/

#[inline]
fn native_call_sudo_raw(
    gas_params: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    native_contract_call(
        gas_params,
        context,
        ty_args,
        arguments,
        4, // code_checksum, store_obj_id, env, msg
        "call_sudo_raw",
        move |instance: &mut Instance<MoveBackendApi, ProxyStorage, MoveBackendQuerier>,
              env: &[u8],
              _info: Option<&[u8]>,
              msg: &[u8]|
              -> VmResult<Vec<u8>> { call_sudo_raw(instance, env, msg) },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: CommonGasParametersOption,
    pub native_create_instance: CosmWasmCreateInstanceGasParametersOption,
    pub native_destroy_instance: CosmWasmDestroyInstanceGasParametersOption,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParametersOption::zeros(),
            native_create_instance: CosmWasmCreateInstanceGasParametersOption::zeros(),
            native_destroy_instance: CosmWasmDestroyInstanceGasParametersOption::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = Vec::new();

    if gas_params.common.load_base.is_some() || gas_params.common.load_per_byte.is_some() {
        natives.push((
            "native_create_instance",
            make_native(gas_params.clone(), native_create_instance),
        ));

        natives.push((
            "native_destroy_instance",
            make_native(gas_params.clone(), native_destroy_instance),
        ));

        natives.push((
            "native_call_instantiate_raw",
            make_native(gas_params.clone(), native_call_instantiate_raw),
        ));

        natives.push((
            "native_call_execute_raw",
            make_native(gas_params.clone(), native_call_execute_raw),
        ));

        natives.push((
            "native_call_query_raw",
            make_native(gas_params.clone(), native_call_query_raw),
        ));

        natives.push((
            "native_call_migrate_raw",
            make_native(gas_params.clone(), native_call_migrate_raw),
        ));

        natives.push((
            "native_call_reply_raw",
            make_native(gas_params.clone(), native_call_reply_raw),
        ));

        natives.push((
            "native_call_sudo_raw",
            make_native(gas_params.clone(), native_call_sudo_raw),
        ));
    }

    make_module_natives(natives)
}
