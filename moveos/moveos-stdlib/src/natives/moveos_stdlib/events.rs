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
use std::sync::Arc;

use crate::natives::helpers;

// pub const MaxEventEmitSize: u64 = 256000;

/***************************************************************************************************
 * native fun event emit
 * Implementation of the Move native function `event::emit<T: copy + drop>(event: T)`
 * Adds an event to the transaction's event log
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct EventEmitGasParameters {
    pub base: InternalGas,
}

impl EventEmitGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

pub fn native_write_to_event_store(
    gas_params: &EventEmitGasParameters,
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

pub fn make_write_to_event_store(gas_params: EventEmitGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_write_to_event_store(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub emit: EventEmitGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            emit: EventEmitGasParameters::zeros(),
        }
    }
}
