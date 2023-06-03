// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{Location, PartialVMError, PartialVMResult};
use move_core_types::{gas_algebra::InternalGas, vm_status::StatusCode};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{StructRef, Value},
};
use smallvec::smallvec;
use std::collections::VecDeque;

use crate::natives::helpers;

// pub const MaxEmitSize: u64 = 256000;

/***************************************************************************************************
 * native fun event emit
 * Implementation of the Move native function `event::emit<T: copy + drop>(event_handle_id: &ObjectID, count: u64, data: T)`
 * Adds an event to the transaction's event log
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct EmitGasParameters {
    pub base: InternalGas,
}

impl EmitGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
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
    debug_assert!(args.len() == 3);

    // TODO(Gas): Charge the arg size dependent costs

    let ty = ty_args.pop().unwrap();
    let msg = args.pop_back().unwrap();
    let seq_num = pop_arg!(args, u64);
    let raw_event_handler_id = pop_arg!(args, StructRef);
    // event_handler_id equal to guid
    let event_handler_id = helpers::get_object_id(raw_event_handler_id)?;

    let _result = context
        .save_event(event_handler_id.to_bytes(), seq_num, ty, msg)
        .map_err(|e| {
            PartialVMError::new(StatusCode::ABORTED)
                .with_message(e.to_string())
                .finish(Location::Undefined)
        });

    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub emit: EmitGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            emit: EmitGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("emit", helpers::make_native(gas_params.emit, native_emit))];
    helpers::make_module_natives(natives)
}
