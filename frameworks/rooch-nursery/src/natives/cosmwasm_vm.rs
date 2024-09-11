// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use log::{debug, warn};
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
use core::slice::SlicePattern;
use std::collections::VecDeque;
use std::ffi::CString;
use std::ops::Deref;
use std::vec;

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use cosmwasm_vm::{Cache, CacheOptions, InstanceOptions, Size, Capability, Checksum};
use moveos_object_runtime::runtime_object::RuntimeObject;
use moveos_object_runtime::TypeLayoutLoader;
use moveos_types::state_resolver::StatelessResolver;
use once_cell::sync::Lazy;
 
use rooch_cosmwasm_vm::backend::{MoveBackendApi, MoveStorage, MoveBackendQuerier, build_move_backend}

use moveos_stdlib::natives::helpers::{make_module_natives, make_native};

use crate::natives::helper::{pop_object_id, CommonGasParameters};

const E_WASM_ERROR: u64 = 1;

fn supported_capabilities() -> HashSet<Capability> {
    let mut capabilities = HashSet::new();
    capabilities.insert(Capability::Iterator);
    capabilities.insert(Capability::Staking);
    capabilities
}

static WASM_CACHE: Lazy<Arc<Cache<MoveBackendApi, MoveStorage, MoveBackendQuerier>>> = Lazy::new(|| {
    let options = CacheOptions::new(
        std::env::temp_dir(),
        supported_capabilities(),
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
    debug_assert!(ty_args.is_empty(), "native_create_instance expects no type arguments");
    debug_assert_eq!(args.len(), 2, "native_create_instance expects 2 arguments");

    let common_gas_parameter = gas_parameters.common.clone();
    let create_instance_gas_parameter = gas_parameters.native_create_instance.clone();

    let wasm_bytes = pop_arg!(args, Vec<u8>);
    let store_obj_id = pop_object_id(&mut args)?;

    wasm_instance_fn_dispatch(
        &common_gas_parameter,
        create_instance_gas_parameter.base,
        create_instance_gas_parameter.per_byte_wasm,
        context,
        store_obj_id,
        wasm_bytes,
        move |layout_loader, resolver, rt_obj, wasm_bytes| -> PartialVMResult<(Value, Option<Option<NumBytes>>)> {
            // Save WASM bytecode and get checksum
            let checksum = WASM_CACHE.save_wasm(wasm_bytes.as_slice())?;

            // Create Backend
            let backend = build_move_backend(Arc::new(Mutex::new(rt_obj.clone())), layout_loader, resolver);

            // Create WASM instance
            let instance_options = InstanceOptions {
                gas_limit: gas_parameters.gas_limit,
            };
            WASM_CACHE.get_instance(&checksum, backend, instance_options)?;

            Ok((Value::vector_u8(checksum.to_vec()), Some(Some(NumBytes::new(wasm_bytes.len() as u64)))))
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
            smallvec![value],
        )),
        Err(err) => {
            let abort_code = error_to_abort_code(err);
            Ok(NativeResult::err(gas_cost, abort_code))
        }
    }
}

// Helper function: convert PartialVMError to abort code
fn error_to_abort_code(err: PartialVMError) -> u64 {
    match err.major_status() {
        StatusCode::ABORTED => err.sub_status().unwrap_or(1),
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
            make_native(gas_params, native_create_instance),
        ),
        (
            "native_destroy_instance",
            make_native(gas_params.native_destroy_instance, native_destroy_instance),
        ),
    ];

    make_module_natives(natives)
}
