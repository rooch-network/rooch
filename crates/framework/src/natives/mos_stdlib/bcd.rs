// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

use crate::natives::helpers::make_module_natives;

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {}
    }
}

/// Rust implementation of Move's `native public(friend) fun from_bytes<T>(vector<u8>): T in bcs module`
/// Bytes are in BCS (Binary Canonical Serialization) format.
#[inline]
fn native_from_bytes(
    _gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let cost = 0.into();

    // TODO(Gas): charge for getting the layout
    let layout = context.type_to_type_layout(&ty_args[0])?.ok_or_else(|| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(format!(
            "Failed to get layout of type {:?} -- this should not happen",
            ty_args[0]
        ))
    })?;

    let bytes = pop_arg!(args, Vec<u8>);
    let val = match Value::simple_deserialize(&bytes, &layout) {
        Some(val) => val,
        None => {
            // TODO(gas): charge the gas for the failure.
            return Err(PartialVMError::new(StatusCode::VALUE_DESERIALIZATION_ERROR));
        }
    };
    // TODO(gas): charge gas for deserialization

    Ok(NativeResult::ok(cost, smallvec![val]))
}

pub fn make_native_from_bytes(gas_params: FromBytesGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_from_bytes(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub from_bytes: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            from_bytes: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("from_bytes", make_native_from_bytes(gas_params.from_bytes))];

    make_module_natives(natives)
}
