// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{error_to_abort_code, CommonGasParameters};
use crate::natives::moveos_stdlib::object::{pop_object_id, read_object_id};
use move_binary_format::errors::PartialVMResult;
use move_core_types::{
    account_address::AccountAddress, gas_algebra::InternalGas, language_storage::TypeTag,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use moveos_object_runtime::{runtime::ObjectRuntimeContext, runtime_object::RuntimeObject};
use moveos_types::moveos_std::object::ObjectID;
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

/***************************************************************************************************
 * native fun native_borrow_object<T: key>(object_id: ObjectID): &Object<T>;
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct BorrowObjectGasParameters {
    pub base: InternalGas,
}

impl BorrowObjectGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

#[inline]
fn native_borrow_object(
    common_gas_params: &CommonGasParameters,
    gas_params: &BorrowObjectGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_fn_dispatch(
        common_gas_params,
        gas_params.base,
        context,
        obj_id,
        &ty_args[0],
        |obj, ty| obj.borrow_object(Some(ty)).map(Some),
    )
}

pub fn make_native_borrow_object(
    common_gas_params: CommonGasParameters,
    gas_params: BorrowObjectGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_borrow_object(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_take_object<T: key>(object_id: ObjectID): Object<T>;
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct TakeObjectGasParameters {
    pub base: InternalGas,
}

impl TakeObjectGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

#[inline]
fn native_take_object(
    common_gas_params: &CommonGasParameters,
    gas_params: &TakeObjectGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let obj_id = pop_object_id(&mut args)?;
    object_fn_dispatch(
        common_gas_params,
        gas_params.base,
        context,
        obj_id,
        &ty_args[0],
        |obj, ty| obj.take_object(Some(ty)).map(Some),
    )
}

pub fn make_native_take_object(
    common_gas_params: CommonGasParameters,
    gas_params: TakeObjectGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_take_object(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_transfer_object<T: key>(obj: Object<T>, new_owner: address);
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct TransferObjectGasParameters {
    pub base: InternalGas,
}

impl TransferObjectGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

#[inline]
fn native_transfer_object(
    common_gas_params: &CommonGasParameters,
    gas_params: &TransferObjectGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 2);
    let new_owner = pop_arg!(args, AccountAddress);
    let obj = args.pop_back().unwrap();
    let obj_id = read_object_id(&obj)?;
    object_fn_dispatch(
        common_gas_params,
        gas_params.base,
        context,
        obj_id,
        &ty_args[0],
        |rt_obj, ty| {
            rt_obj
                .transfer_object(obj, new_owner, Some(ty))
                .map(|_| None)
        },
    )
}

pub fn make_native_transfer_object(
    common_gas_params: CommonGasParameters,
    gas_params: TransferObjectGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_transfer_object(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_to_shared_object<T: key>(obj: Object<T>);
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct ToSharedObjectGasParameters {
    pub base: InternalGas,
}

impl ToSharedObjectGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

#[inline]
fn native_to_shared_object(
    common_gas_params: &CommonGasParameters,
    gas_params: &ToSharedObjectGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);
    let obj = args.pop_back().unwrap();
    let obj_id = read_object_id(&obj)?;
    object_fn_dispatch(
        common_gas_params,
        gas_params.base,
        context,
        obj_id,
        &ty_args[0],
        |rt_obj, ty| rt_obj.to_shared_object(obj, Some(ty)).map(|_| None),
    )
}

pub fn make_native_to_shared_object(
    common_gas_params: CommonGasParameters,
    gas_params: ToSharedObjectGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_to_shared_object(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun native_to_frozen_object<T: key>(obj: Object<T>);
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct ToFrozenObjectGasParameters {
    pub base: InternalGas,
}

impl ToFrozenObjectGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

#[inline]
fn native_to_frozen_object(
    common_gas_params: &CommonGasParameters,
    gas_params: &ToFrozenObjectGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);
    let obj = args.pop_back().unwrap();
    let obj_id = read_object_id(&obj)?;
    object_fn_dispatch(
        common_gas_params,
        gas_params.base,
        context,
        obj_id,
        &ty_args[0],
        |rt_obj, ty| rt_obj.to_frozen_object(obj, Some(ty)).map(|_| None),
    )
}

pub fn make_native_to_frozen_object(
    common_gas_params: CommonGasParameters,
    gas_params: ToFrozenObjectGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_to_frozen_object(&common_gas_params, &gas_params, context, ty_args, args)
    })
}

#[inline]
fn object_fn_dispatch(
    common_gas_params: &CommonGasParameters,
    base: InternalGas,
    context: &mut NativeContext,
    obj_id: ObjectID,
    ref_type: &Type,
    f: impl FnOnce(&mut RuntimeObject, &TypeTag) -> PartialVMResult<Option<Value>>,
) -> PartialVMResult<NativeResult> {
    let type_tag = context.type_to_type_tag(ref_type)?;
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();
    let (object, object_load_gas) =
        object_runtime.load_object(context, object_context.resolver(), &obj_id)?;
    let gas_cost = base + common_gas_params.calculate_load_cost(object_load_gas);
    let result = f(object, &type_tag);
    match result {
        Ok(ret) => Ok(NativeResult::ok(
            gas_cost,
            ret.map(|v| smallvec![v]).unwrap_or(smallvec![]),
        )),
        Err(err) => {
            let abort_code = error_to_abort_code(err);
            Ok(NativeResult::err(gas_cost, abort_code))
        }
    }
}
