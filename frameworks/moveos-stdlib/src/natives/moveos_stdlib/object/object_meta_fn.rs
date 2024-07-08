// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{error_to_abort_code, CommonGasParameters};
use crate::natives::moveos_stdlib::object::pop_object_id;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use moveos_object_runtime::{runtime::ObjectRuntimeContext, runtime_object::RuntimeObject};
use moveos_types::moveos_std::object::ObjectID;
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

/// All ObjectMeta functions use the same gas parameters
#[derive(Debug, Clone)]
pub struct ObjectMetaGasParameters {
    pub base: InternalGas,
}

impl ObjectMetaGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun native_object_owner(obj_id: ObjectID): address;
 **************************************************************************************************/

#[inline]
fn native_object_owner(
    common_gas_params: &CommonGasParameters,
    gas_params: &ObjectMetaGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_meta_fn_dispatch(common_gas_params, gas_params.base, context, obj_id, |obj| {
        Ok(Value::address(obj.metadata()?.owner))
    })
}

pub fn make_native_object_owner(
    common_gas_params: CommonGasParameters,
    gas_params: ObjectMetaGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_object_owner(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_object_size(obj_id: ObjectID): u64;
 **************************************************************************************************/

#[inline]
fn native_object_size(
    common_gas_params: &CommonGasParameters,
    gas_params: &ObjectMetaGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_meta_fn_dispatch(common_gas_params, gas_params.base, context, obj_id, |obj| {
        Ok(Value::u64(obj.metadata()?.size))
    })
}

pub fn make_native_object_size(
    common_gas_params: CommonGasParameters,
    gas_params: ObjectMetaGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_object_size(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_object_flag(obj_id: ObjectID): u8;
 **************************************************************************************************/

#[inline]
fn native_object_flag(
    common_gas_params: &CommonGasParameters,
    gas_params: &ObjectMetaGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_meta_fn_dispatch(common_gas_params, gas_params.base, context, obj_id, |obj| {
        Ok(Value::u8(obj.metadata()?.flag))
    })
}

pub fn make_native_object_flag(
    common_gas_params: CommonGasParameters,
    gas_params: ObjectMetaGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_object_flag(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_object_created_at(obj_id: ObjectID): u64;
 **************************************************************************************************/

#[inline]
fn native_object_created_at(
    common_gas_params: &CommonGasParameters,
    gas_params: &ObjectMetaGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_meta_fn_dispatch(common_gas_params, gas_params.base, context, obj_id, |obj| {
        Ok(Value::u64(obj.metadata()?.created_at))
    })
}

pub fn make_native_object_created_at(
    common_gas_params: CommonGasParameters,
    gas_params: ObjectMetaGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_object_created_at(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_object_updated_at(obj_id: ObjectID): u64;
 **************************************************************************************************/

#[inline]
fn native_object_updated_at(
    common_gas_params: &CommonGasParameters,
    gas_params: &ObjectMetaGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_meta_fn_dispatch(common_gas_params, gas_params.base, context, obj_id, |obj| {
        Ok(Value::u64(obj.metadata()?.updated_at))
    })
}

pub fn make_native_object_updated_at(
    common_gas_params: CommonGasParameters,
    gas_params: ObjectMetaGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_object_updated_at(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

fn object_meta_fn_dispatch(
    common_gas_params: &CommonGasParameters,
    base: InternalGas,
    context: &mut NativeContext,
    obj_id: ObjectID,
    f: impl FnOnce(&mut RuntimeObject) -> PartialVMResult<Value>,
) -> PartialVMResult<NativeResult> {
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();
    let (object, object_load_gas) =
        object_runtime.load_object(context, object_context.resolver(), &obj_id)?;
    let gas_cost = base + common_gas_params.calculate_load_cost(object_load_gas);
    let result = f(object);
    match result {
        Ok(ret) => Ok(NativeResult::ok(gas_cost, smallvec![ret])),
        Err(err) => {
            let abort_code = error_to_abort_code(err);
            Ok(NativeResult::err(gas_cost, abort_code))
        }
    }
}
