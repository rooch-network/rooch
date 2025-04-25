// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers;
use crate::natives::moveos_stdlib::object::pop_object_id;
use better_any::{Tid, TidAble};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    language_storage::{StructTag, TypeTag},
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use moveos_types::moveos_std::event::EventHandle;
use moveos_types::moveos_std::object::ObjectID;
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Default, Tid)]
pub struct NativeEventContext {
    events: Vec<(StructTag, ObjectID, Vec<u8>)>,
}

impl NativeEventContext {
    pub fn into_events(self) -> Vec<(StructTag, ObjectID, Vec<u8>)> {
        self.events
    }
}

// pub const MaxEmitSize: u64 = 256000;

/***************************************************************************************************
 * native fun event emit
 * Implementation of the Move native function `event::emit<T: copy + drop>(event_handle_id: &ObjectID, count: u64, data: T)`
 * Adds an event to the transaction's event log
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct EmitGasParameters {
    pub base: InternalGas,
    pub per_byte_in_str: InternalGasPerByte,
}

impl EmitGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
            per_byte_in_str: InternalGasPerByte::zero(),
        }
    }
}

pub fn native_emit(
    gas_params: &EmitGasParameters,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let mut cost = gas_params.base;

    let ty = ty_args.pop().unwrap();
    let type_tag = context.type_to_type_tag(&ty)?;
    let struct_tag = match type_tag {
        TypeTag::Struct(struct_tag) => *struct_tag,
        _ => {
            debug_assert!(false, "Event type should be a struct");
            return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR));
        }
    };
    let msg = args.pop_back().unwrap();
    let layout = match context.type_to_type_layout(&ty) {
        Ok(layout) => layout,
        Err(_) => {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "Failed to get layout of type {:?} -- this should not happen",
                        ty
                    ),
                ),
            )
        }
    };

    if tracing::enabled!(tracing::Level::TRACE) {
        tracing::trace!("Emitting event {}, {:?}", struct_tag, msg);
    }
    let event_data = msg
        .simple_serialize(&layout)
        .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
    cost += gas_params.per_byte_in_str * NumBytes::new(event_data.len() as u64);

    let event_handle_id = EventHandle::derive_event_handle_id(&struct_tag);
    let event_context = context.extensions_mut().get_mut::<NativeEventContext>();
    event_context
        .events
        .push((struct_tag, event_handle_id, event_data));

    Ok(NativeResult::ok(cost, smallvec![]))
}

#[derive(Debug, Clone)]
pub struct EmitWithHandleGasParameters {
    pub base: Option<InternalGas>,
    pub per_byte_in_str: Option<InternalGasPerByte>,
}

impl EmitWithHandleGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: Some(InternalGas::zero()),
            per_byte_in_str: Some(InternalGasPerByte::zero()),
        }
    }

    pub fn init(base: InternalGas, per_byte: InternalGasPerByte) -> Self {
        Self {
            base: Some(base),
            per_byte_in_str: Some(per_byte),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base.is_none() || self.per_byte_in_str.is_none()
    }
}

pub fn native_emit_with_handle(
    gas_params: &EmitWithHandleGasParameters,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 2);

    let mut cost = gas_params.base.unwrap_or_else(InternalGas::zero);

    let ty = ty_args.pop().unwrap();
    let type_tag = context.type_to_type_tag(&ty)?;
    let struct_tag = match type_tag {
        TypeTag::Struct(struct_tag) => *struct_tag,
        _ => {
            debug_assert!(false, "Event type should be a struct");
            return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR));
        }
    };

    let msg = args.pop_back().unwrap();
    let layout = match context.type_to_type_layout(&ty) {
        Ok(v) => v,
        Err(_) => {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "Failed to get layout of type {:?} -- this should not happen",
                        ty
                    ),
                ),
            )
        }
    };
    if tracing::enabled!(tracing::Level::TRACE) {
        tracing::trace!("Emitting event {}, {:?}", struct_tag, msg);
    }
    let event_data = msg
        .simple_serialize(&layout)
        .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
    cost += gas_params
        .per_byte_in_str
        .unwrap_or_else(InternalGasPerByte::zero)
        * NumBytes::new(event_data.len() as u64);

    let event_handle_id = pop_object_id(&mut args)?;
    let event_context = context.extensions_mut().get_mut::<NativeEventContext>();
    event_context
        .events
        .push((struct_tag, event_handle_id, event_data));

    Ok(NativeResult::ok(cost, smallvec![]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub emit: EmitGasParameters,
    pub emit_with_handle: EmitWithHandleGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            emit: EmitGasParameters::zeros(),
            emit_with_handle: EmitWithHandleGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "native_emit",
            helpers::make_native(gas_params.emit, native_emit),
        ),
        (
            "native_emit_with_handle",
            helpers::make_native(gas_params.emit_with_handle, native_emit_with_handle),
        ),
    ];

    helpers::make_module_natives(natives)
}
