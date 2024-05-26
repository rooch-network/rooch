// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use revm_precompile::bn128;
use smallvec::smallvec;
use std::collections::VecDeque;

pub const E_EC_ADD_FAILED: u64 = 6;
pub const E_EC_PAIRING_FAILED: u64 = 8;
pub const E_INVALID_COORDINATE: u64 = 11;

/***************************************************************************************************
 * native fun ec_add
 * Implementation of the Move native function `ec_add(x1: vector<u8>, y1: vector<u8>, x2: vector<u8>, y2: vector<u8>): (vector<u8>, vector<u8>)`
 *   gas cost: ec_add_cost_base                               | base cost for function call and fixed opers
 *              + ec_add_data_cost_per_byte * 128       | cost depends on length of message
 *              + ec_add_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_ec_add(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 4);

    let y2 = pop_arg!(args, Vec<u8>);
    let x2 = pop_arg!(args, Vec<u8>);
    let y1 = pop_arg!(args, Vec<u8>);
    let x1 = pop_arg!(args, Vec<u8>);

    if y2.len() != 32 || x2.len() != 32 || y1.len() != 32 || x1.len() != 32 {
        return Ok(NativeResult::err(0.into(), E_INVALID_COORDINATE));
    }

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(128_u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(x1);
    data.extend(y1);
    data.extend(x2);
    data.extend(y2);

    match bn128::run_add(&data, 0, 10) {
        Ok((_, sum)) => {
            let (sum_x, sum_y) = sum.split_at(32);
            let sum_x = Value::vector_u8(sum_x.to_vec());
            let sum_y = Value::vector_u8(sum_y.to_vec());
            Ok(NativeResult::ok(cost, smallvec![sum_x, sum_y]))
        }

        Err(_) => Ok(NativeResult::err(cost, E_EC_ADD_FAILED)),
    }
}

/***************************************************************************************************
 * native fun ec_pairing
 * Implementation of the Move native function `ec_pairing(data: vector<u8>): vector<u8>`
 *   gas cost: ec_pairing_cost_base                               | base cost for function call and fixed opers
 *              + ec_pairing_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + ec_pairing_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_ec_pairing(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let data = pop_arg!(args, Vec<u8>);

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(data.len() as u64));

    match bn128::run_pair(&data, 0, 0, 10) {
        Ok((_, pair)) => {
            let success = Value::vector_u8(pair.to_vec());
            Ok(NativeResult::ok(cost, smallvec![success]))
        }
        Err(_) => Ok(NativeResult::err(cost, E_EC_PAIRING_FAILED)),
    }
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub ec_add: FromBytesGasParameters,
    pub ec_pairing: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            ec_add: FromBytesGasParameters::zeros(),
            ec_pairing: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("ec_add", make_native(gas_params.ec_add, native_ec_add)),
        (
            "ec_pairing",
            make_native(gas_params.ec_pairing, native_ec_pairing),
        ),
    ];

    make_module_natives(natives)
}
