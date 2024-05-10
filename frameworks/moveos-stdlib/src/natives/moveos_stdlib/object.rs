// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::{helpers::make_module_natives, helpers::make_native};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use moveos_object_runtime::runtime::{ObjectRuntimeContext, RuntimeField, TypeLayoutLoader};
use moveos_types::{
    moveos_std::object::{Object, ObjectID},
    state::{KeyState, MoveState, PlaceholderStruct},
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

pub use moveos_object_runtime::runtime::{
    ERROR_ALREADY_EXISTS, ERROR_NOT_FOUND, ERROR_OBJECT_ALREADY_BORROWED,
    ERROR_OBJECT_RUNTIME_ERROR, ERROR_TYPE_MISMATCH,
};

#[derive(Debug, Clone)]
pub struct AsRefGasParameters {
    pub base: InternalGas,
}

impl AsRefGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun as_ref_inner
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_as_ref_inner(
    gas_params: &AsRefGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.len() == 1);

    let object_id = arguments.pop_back().unwrap();
    let object_ref = borrow_object_reference(context, object_id, &ty_args[0])?;
    Ok(NativeResult::ok(gas_params.base, smallvec![object_ref]))
}

pub fn make_native_as_ref_inner(gas_params: AsRefGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| native_as_ref_inner(&gas_params, context, ty_args, args))
}

#[derive(Debug, Clone)]
pub struct AsMutRefGasParameters {
    pub base: InternalGas,
}

impl AsMutRefGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun as_mut_ref_inner
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_as_mut_ref_inner(
    gas_params: &AsMutRefGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.len() == 1);

    let object_id = arguments.pop_back().unwrap();
    let object_ref = borrow_object_reference(context, object_id, &ty_args[0])?;
    Ok(NativeResult::ok(gas_params.base, smallvec![object_ref]))
}

pub fn make_native_as_mut_ref_inner(gas_params: AsMutRefGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_as_mut_ref_inner(&gas_params, context, ty_args, args)
    })
}

fn borrow_object_reference(
    context: &mut NativeContext,
    object_id_value: Value,
    ref_type: &Type,
) -> PartialVMResult<Value> {
    let type_tag = context.type_to_type_tag(ref_type)?;
    let type_layout = context
        .get_type_layout(&type_tag)
        .map_err(|e| e.to_partial())?;

    debug_assert!(
        Object::<PlaceholderStruct>::type_layout_match(&type_layout),
        "Expected a struct type with layout same as Object<T>"
    );

    let object_id = ObjectID::from_runtime_value(object_id_value)
        .map_err(|_e| partial_extension_error("Invalid object id argument"))?;
    let object_context = context.extensions_mut().get_mut::<ObjectRuntimeContext>();

    let data = object_context.object_runtime();
    let mut object_runtime = data.write();
    //TODO remove load_object, the object should loaded when load ObjectEntity
    object_runtime
        .load_object_reference(&object_id)
        .map_err(|e| e.to_partial())?;
    object_runtime
        .borrow_object_reference(&object_id)
        .map_err(|e| e.to_partial())
}

#[derive(Debug, Clone)]
pub struct CommonGasParameters {
    pub load_base: InternalGas,
    pub load_per_byte: InternalGasPerByte,
    pub load_failure: InternalGas,
}

impl CommonGasParameters {
    fn calculate_load_cost(&self, loaded: Option<Option<NumBytes>>) -> InternalGas {
        self.load_base
            + match loaded {
                Some(Some(num_bytes)) => self.load_per_byte * num_bytes,
                Some(None) => self.load_failure,
                None => 0.into(),
            }
    }
}

fn native_fn_dispatch(
    common_gas_params: &CommonGasParameters,
    base: InternalGas,
    per_byte_serialized: InternalGasPerByte,
    context: &mut NativeContext,
    object_id: ObjectID,
    field_key: KeyState,
    f: impl FnOnce(&dyn TypeLayoutLoader, &mut RuntimeField) -> PartialVMResult<Option<Value>>,
) -> PartialVMResult<NativeResult> {
    log::debug!("native_fn_dispatch 1");

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let binding = object_context.object_runtime();
    let mut object_runtime = binding.write();

    log::debug!("native_fn_dispatch 2");

    let (object, object_load_gas) =
        object_runtime.load_object(context, object_context.resolver(), &object_id)?;
        log::debug!("native_fn_dispatch 3");
    let field_key_bytes = field_key.key.len() as u64;
    let (tv, field_load_gas) =
        object.load_field(context, object_context.resolver(), field_key.clone())?;
        log::debug!("native_fn_dispatch 4");
    let gas_cost = base
        + per_byte_serialized * NumBytes::new(field_key_bytes)
        + common_gas_params.calculate_load_cost(object_load_gas)
        + common_gas_params.calculate_load_cost(field_load_gas);

    log::debug!("native_fn_dispatch 5");

    let result = f(context, tv);
    match result {
        Ok(ret) => Ok(NativeResult::ok(
            gas_cost,
            ret.map(|v| smallvec![v]).unwrap_or(smallvec![]),
        )),
        Err(err) => {
            log::debug!("native_fn_dispatch err 1");

            let abort_code = match err.major_status() {
                StatusCode::MISSING_DATA => ERROR_NOT_FOUND,
                StatusCode::TYPE_MISMATCH => ERROR_TYPE_MISMATCH,
                StatusCode::RESOURCE_ALREADY_EXISTS => ERROR_ALREADY_EXISTS,
                _ => ERROR_OBJECT_RUNTIME_ERROR,
            };
            if log::log_enabled!(log::Level::Debug) {
                log::warn!(
                    "[ObjectRuntime] native_function error: object_id: {:?}, key:{:?}, err: {:?}, abort: {}",
                    object_id,
                    field_key,
                    err,
                    abort_code
                );
            };
            Ok(NativeResult::err(gas_cost, abort_code))
        }
    }
}

fn native_borrow_root(
    common_gas_params: &CommonGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 0);
    debug_assert_eq!(args.len(), 0);
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let object_runtime = object_context.object_runtime();
    let value = object_runtime.read().borrow_root()?;
    let gas_cost = common_gas_params.load_base;
    Ok(NativeResult::ok(gas_cost, smallvec![value]))
}

pub fn make_native_borrow_root(common_gas_params: CommonGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_borrow_root(&common_gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct AddFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_add_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &AddFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    //0 K Type
    //1 V Type FieldValue or ObjectEntity

    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 3);

    let val = args.pop_back().unwrap();
    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        move |layout_loader, field| {
            let value_layout = layout_loader.type_to_type_layout(&ty_args[1])?;
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            field.move_to(val, value_layout, value_type).map(|_| None)
        },
    )
}

pub fn make_native_add_field(
    common_gas_params: CommonGasParameters,
    gas_params: AddFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_add_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct BorrowFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_borrow_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &BorrowFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |layout_loader, field| {
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            field.borrow_value(value_type).map(Some)
        },
    )
}

pub fn make_native_borrow_field(
    common_gas_params: CommonGasParameters,
    gas_params: BorrowFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_borrow_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct ContainsFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_contains_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &ContainsFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |_layout_loader, field| Ok(Some(Value::bool(field.exists()?))),
    )
}

pub fn make_native_contains_field(
    common_gas_params: CommonGasParameters,
    gas_params: ContainsFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_contains_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

fn native_contains_field_with_value_type(
    common_gas_params: &CommonGasParameters,
    gas_params: &ContainsFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |layout_loader, field| {
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            Ok(Some(Value::bool(field.exists_with_type(value_type)?)))
        },
    )
}

pub fn make_native_contains_field_with_value_type(
    common_gas_params: CommonGasParameters,
    gas_params: ContainsFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_contains_field_with_value_type(
                &common_gas_params,
                &gas_params,
                context,
                ty_args,
                args,
            )
        },
    )
}

#[derive(Debug, Clone)]
pub struct RemoveFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_remove_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &RemoveFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |layout_loader, field| {
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            field.move_from(value_type).map(Some)
        },
    )
}

pub fn make_native_remove_field(
    common_gas_params: CommonGasParameters,
    gas_params: RemoveFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_remove_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

// =========================================================================================
// Helpers

fn pop_object_id(args: &mut VecDeque<Value>) -> PartialVMResult<ObjectID> {
    let handle = args.pop_back().unwrap();
    ObjectID::from_runtime_value(handle).map_err(|e| {
        if log::log_enabled!(log::Level::Debug) {
            log::warn!("[ObjectRuntime] get_object_id: {:?}", e);
        }
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })
}

fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    log::debug!("PartialVMError: {}", msg.to_string());
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}

fn type_to_type_layout(context: &NativeContext, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
    context
        .type_to_type_layout(ty)?
        .ok_or_else(|| partial_extension_error("cannot determine type layout"))
}

fn serialize_key(
    context: &NativeContext,
    key_type: &Type,
    key: Value,
) -> PartialVMResult<KeyState> {
    let key_layout = type_to_type_layout(context, key_type)?;
    let key_type_tag = context.type_to_type_tag(key_type)?;
    let key_bytes = moveos_object_runtime::runtime::serialize(&key_layout, &key)?;
    Ok(KeyState::new(key_bytes, key_type_tag))
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: CommonGasParameters,
    pub as_ref_inner: AsRefGasParameters,
    pub as_mut_ref_inner: AsMutRefGasParameters,
    pub native_add_field: AddFieldGasParameters,
    pub native_borrow_field: BorrowFieldGasParameters,
    pub native_contains_field: ContainsFieldGasParameters,
    pub native_contains_field_with_value_type: ContainsFieldGasParameters,
    pub native_remove_field: RemoveFieldGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParameters {
                load_base: 0.into(),
                load_per_byte: 0.into(),
                load_failure: 0.into(),
            },
            as_ref_inner: AsRefGasParameters::zeros(),
            as_mut_ref_inner: AsMutRefGasParameters::zeros(),
            native_add_field: AddFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_borrow_field: BorrowFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_contains_field: ContainsFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_contains_field_with_value_type: ContainsFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            native_remove_field: RemoveFieldGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "as_ref_inner",
            make_native(gas_params.as_ref_inner, native_as_ref_inner),
        ),
        (
            "as_mut_ref_inner",
            make_native(gas_params.as_mut_ref_inner, native_as_mut_ref_inner),
        ),
        (
            "native_borrow_root",
            make_native_borrow_root(gas_params.common.clone()),
        ),
        (
            "native_add_field",
            make_native_add_field(gas_params.common.clone(), gas_params.native_add_field),
        ),
        (
            "native_borrow_field",
            make_native_borrow_field(
                gas_params.common.clone(),
                gas_params.native_borrow_field.clone(),
            ),
        ),
        (
            "native_borrow_mut_field",
            make_native_borrow_field(gas_params.common.clone(), gas_params.native_borrow_field),
        ),
        (
            "native_remove_field",
            make_native_remove_field(gas_params.common.clone(), gas_params.native_remove_field),
        ),
        (
            "native_contains_field",
            make_native_contains_field(gas_params.common.clone(), gas_params.native_contains_field),
        ),
        (
            "native_contains_field_with_value_type",
            make_native_contains_field_with_value_type(
                gas_params.common,
                gas_params.native_contains_field_with_value_type,
            ),
        ),
    ];

    make_module_natives(natives)
}
