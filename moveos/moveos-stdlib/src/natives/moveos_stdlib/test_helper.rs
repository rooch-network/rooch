// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::make_module_natives;

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

#[derive(Debug, Clone)]
pub struct DestroyGasParameters {
    pub base: InternalGas,
}

fn native_destroy(
    gas_params: &DestroyGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(ty_args.len(), 1);
    assert_eq!(args.len(), 1);

    args.pop_back();
    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

pub fn make_native_destroy(gas_params: DestroyGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_destroy(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub destroy: DestroyGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            destroy: DestroyGasParameters { base: 0.into() },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("destroy", make_native_destroy(gas_params.destroy))];

    make_module_natives(natives)
}
