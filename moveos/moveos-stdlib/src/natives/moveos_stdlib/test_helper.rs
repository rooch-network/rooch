// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives};

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::{
    native_functions::{NativeContext, NativeFunction},
};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value, pop_arg
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};
use move_core_types::vm_status::StatusCode;

#[derive(Debug, Clone)]
pub struct DropUncheckedGasParameters {
    pub base: InternalGas,
}

fn native_drop_unchecked_box(
    gas_params: &DropUncheckedGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(ty_args.len(), 1);
    assert_eq!(args.len(), 1);

    let bytes = pop_arg!(args, Vec<u8>);
    println!("{:?} Native test helper bytes .", bytes);
    // TODO(Gas): charge for getting the layout
    let layout = context.type_to_type_layout(&ty_args[0])?.ok_or_else(|| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(format!(
            "Failed to get layout of type {:?} -- this should not happen",
            ty_args[0]
        ))
    })?;
    println!("{:?} Native test helper layout .", layout);

    let bytes = pop_arg!(args, Vec<u8>);
    let val = match Value::simple_deserialize(&bytes, &layout) {
        Some(val) => val,
        None => {
            // TODO(gas): charge the gas for the failure.
            return Err(PartialVMError::new(StatusCode::VALUE_DESERIALIZATION_ERROR));
        }
    };
    println!("{:?} Native test helper val .", val);

    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

pub fn make_native_drop_unchecked(gas_params: DropUncheckedGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_drop_unchecked_box(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub drop_unchecked: DropUncheckedGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            drop_unchecked: DropUncheckedGasParameters { base: 0.into() },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "drop_unchecked",
        make_native_drop_unchecked(gas_params.drop_unchecked),
    )];

    make_module_natives(natives)
}
