// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use move_binary_format::errors::PartialVMResult;
use move_core_types::{
    account_address::AccountAddress,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
};
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::smallvec;

use moveos_object_runtime::{
    runtime::ObjectRuntimeContext, runtime_object::RuntimeObject, TypeLayoutLoader,
};
use moveos_types::{
    moveos_std::object::ObjectID, state::FieldKey, state_resolver::StatelessResolver,
};

use crate::natives::moveos_stdlib::object::{pop_object_id, GasParameters};

use super::{error_to_abort_code, CommonGasParameters};

/***************************************************************************************************
 * native fun native_add_field<V>(obj_id: ObjectID, key: address, val: V): Object<V>;
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct AddFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

#[inline]
pub(crate) fn native_add_field(
    gas_parameters: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    //0 Value Type: DynamicField or T of Object<T>

    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 3);

    let common_gas_parameter = gas_parameters.common.clone();
    let add_field_gas_parameter = gas_parameters.native_add_field.clone();

    let value = args.pop_back().unwrap();
    let field_key: FieldKey = pop_arg!(args, AccountAddress).into();
    let obj_id = pop_object_id(&mut args)?;

    object_field_fn_dispatch(
        &common_gas_parameter,
        add_field_gas_parameter.base,
        add_field_gas_parameter.per_byte_serialized,
        context,
        obj_id,
        move |layout_loader, resolver, rt_obj| {
            rt_obj.add_field(layout_loader, resolver, field_key, &ty_args[0], value)
        },
    )
}

/***************************************************************************************************
 * native fun native_borrow_field<V>(obj_id: ObjectID, key: address): &V;
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct BorrowFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

#[inline]
pub(crate) fn native_borrow_field(
    gas_parameters: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 2);

    let field_key: FieldKey = pop_arg!(args, AccountAddress).into();
    let obj_id = pop_object_id(&mut args)?;

    let common_gas_parameter = gas_parameters.common.clone();
    let borrow_field_gas_parameter = gas_parameters.native_borrow_field.clone();

    object_field_fn_dispatch(
        &common_gas_parameter,
        borrow_field_gas_parameter.base,
        borrow_field_gas_parameter.per_byte_serialized,
        context,
        obj_id,
        |layout_loader, resolver, rt_obj| {
            rt_obj.borrow_field(layout_loader, resolver, field_key, &ty_args[0])
        },
    )
}

/***************************************************************************************************
 * native fun native_contains_field(obj_id: ObjectID, key: address): bool;
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct ContainsFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

pub(crate) fn native_contains_field(
    gas_parameters: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 0);
    debug_assert_eq!(args.len(), 2);

    let field_key: FieldKey = pop_arg!(args, AccountAddress).into();
    let obj_id = pop_object_id(&mut args)?;

    let common_gas_parameter = gas_parameters.common.clone();
    let contains_field_gas_parameter = gas_parameters.native_contains_field.clone();

    object_field_fn_dispatch(
        &common_gas_parameter,
        contains_field_gas_parameter.base,
        contains_field_gas_parameter.per_byte_serialized,
        context,
        obj_id,
        |layout_loader, resolver, rt_obj| {
            let (rt_field, loaded_gas) = rt_obj.load_field(layout_loader, resolver, field_key)?;
            Ok((Value::bool(rt_field.exists()?), loaded_gas))
        },
    )
}

/***************************************************************************************************
 * native fun native_contains_field_with_value_type<V>(obj_id: ObjectID, key: address): bool;
 **************************************************************************************************/

pub(crate) fn native_contains_field_with_value_type(
    gas_parameters: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 2);

    let field_key: FieldKey = pop_arg!(args, AccountAddress).into();
    let obj_id = pop_object_id(&mut args)?;

    let common_gas_parameter = gas_parameters.common.clone();
    let contains_field_gas_parameter = gas_parameters.native_contains_field_with_value_type.clone();

    object_field_fn_dispatch(
        &common_gas_parameter,
        contains_field_gas_parameter.base,
        contains_field_gas_parameter.per_byte_serialized,
        context,
        obj_id,
        |layout_loader, resolver, rt_obj| {
            let (rt_field, loaded_gas) = rt_obj.load_field(layout_loader, resolver, field_key)?;
            let value_type = layout_loader.type_to_type_tag(&ty_args[0])?;
            Ok((
                Value::bool(rt_field.exists_with_type(&value_type)?),
                loaded_gas,
            ))
        },
    )
}

/***************************************************************************************************
 * native fun native_remove_field<V>(obj_id: ObjectID, key: address): V;
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct RemoveFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

pub(crate) fn native_remove_field(
    gas_parameters: &GasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 2);

    let field_key: FieldKey = pop_arg!(args, AccountAddress).into();
    let obj_id = pop_object_id(&mut args)?;

    let common_gas_parameter = gas_parameters.common.clone();
    let remove_field_gas_parameter = gas_parameters.native_remove_field.clone();

    object_field_fn_dispatch(
        &common_gas_parameter,
        remove_field_gas_parameter.base,
        remove_field_gas_parameter.per_byte_serialized,
        context,
        obj_id,
        |layout_loader, resolver, rt_obj| {
            rt_obj.remove_field(layout_loader, resolver, field_key, &ty_args[0])
        },
    )
}

fn object_field_fn_dispatch(
    common_gas_params: &CommonGasParameters,
    base: InternalGas,
    per_byte_serialized: InternalGasPerByte,
    context: &mut NativeContext,
    obj_id: ObjectID,
    f: impl FnOnce(
        &dyn TypeLayoutLoader,
        &dyn StatelessResolver,
        &mut RuntimeObject,
    ) -> PartialVMResult<(Value, Option<Option<NumBytes>>)>,
) -> PartialVMResult<NativeResult> {
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();
    let resolver = object_runtime.resolver();
    let (rt_obj, object_load_gas) = object_runtime.load_object(context, &obj_id)?;
    let field_key_bytes = AccountAddress::LENGTH as u64;
    let gas_cost = base
        + per_byte_serialized * NumBytes::new(field_key_bytes)
        + common_gas_params.calculate_load_cost(object_load_gas);

    let result = f(context, resolver, rt_obj);
    match result {
        Ok((value, field_load_gas)) => Ok(NativeResult::ok(
            gas_cost + common_gas_params.calculate_load_cost(field_load_gas),
            smallvec![value],
        )),
        Err(err) => {
            let abort_code = error_to_abort_code(err);
            Ok(NativeResult::err(gas_cost, abort_code))
        }
    }
}
