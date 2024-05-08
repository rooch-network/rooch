// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::{Value, Struct, Vector},
};
use smallvec::smallvec;
use std::collections::VecDeque;
use log::debug;

const E_TYPE_NOT_MATCH: u64 = 1;

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte_deserialize: InternalGasPerByte,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte_deserialize: 0.into(),
        }
    }
}

/// Rust implementation of Move's `native public(friend) fun from_bytes<T>(vector<u8>): T in bcs module`
/// Bytes are in BCS (Binary Canonical Serialization) format.
#[inline]
fn native_from_bytes(
    gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let mut cost = gas_params.base;
    let type_param = &ty_args[0];

    // TODO(Gas): charge for getting the layout
    let layout = context.type_to_type_layout(&ty_args[0])?.ok_or_else(|| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(format!(
            "Failed to get layout of type {:?} -- this should not happen",
            ty_args[0]
        ))
    })?;

    let bytes = pop_arg!(args, Vec<u8>);
    cost += gas_params.per_byte_deserialize * NumBytes::new(bytes.len() as u64);
    let result = match Value::simple_deserialize(&bytes, &layout) {
        Some(val) => {
            Struct::pack(vec![Vector::pack(type_param, vec![val]).map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("Failed to pack Option: {:?}", e))
            })?])
        },
        None => {
            // Pack the MoveOption None
            Struct::pack(vec![Vector::pack(type_param, vec![]).map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("Failed to pack Option: {:?}", e))
            })?])
        }
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::struct_(result)],
    ))
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
    let natives = [(
        "native_from_bytes",
        make_native(gas_params.from_bytes, native_from_bytes),
    )];

    make_module_natives(natives)
}
