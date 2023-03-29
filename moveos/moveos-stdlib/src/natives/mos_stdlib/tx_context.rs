// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Source from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/src/natives/tx_context.rs
// and do some refactor

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use moveos_types::object::ObjectID;
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

use crate::natives::helpers::make_module_natives;

#[derive(Debug, Clone)]
pub struct DeriveIDGasParameters {
    pub base: InternalGas,
}

impl DeriveIDGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

//TODO merge this native function with table::new_table_handle
pub fn native_derive_id(
    gas_params: &DeriveIDGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let ids_created = pop_arg!(args, u64);
    let tx_hash = pop_arg!(args, Vec<u8>);

    let object_id = ObjectID::derive_id(tx_hash, ids_created);
    let cost = gas_params.base;
    Ok(NativeResult::ok(
        cost,
        smallvec![Value::address(object_id.into())],
    ))
}

pub fn make_native_derive_id(gas_params: DeriveIDGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_derive_id(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub derive_id: DeriveIDGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            derive_id: DeriveIDGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("derive_id", make_native_derive_id(gas_params.derive_id))];

    make_module_natives(natives)
}
