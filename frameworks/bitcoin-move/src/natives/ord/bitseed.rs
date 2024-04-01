// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use ciborium::value::Integer;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use smallvec::smallvec;

use moveos_stdlib::natives::helpers::{make_module_natives, make_native};

#[derive(Debug, Clone)]
pub struct ArgsPackingGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl ArgsPackingGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/// Rust implementation of native_pack_inscribe_generate_args
#[inline]
pub fn native_pack_inscribe_generate_args(
    gas_params: &ArgsPackingGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let user_input_key = pop_arg!(args, Vec<u8>);
    let user_input = pop_arg!(args, Vec<u8>);
    let seed_key = pop_arg!(args, Vec<u8>);
    let seed = pop_arg!(args, Vec<u8>);
    let deploy_args_key = pop_arg!(args, Vec<u8>);
    let deploy_args = pop_arg!(args, Vec<u8>);

    let user_input_key_string = String::from_utf8_lossy(user_input_key.as_slice()).to_string();
    let seed_key_string = String::from_utf8_lossy(seed_key.as_slice()).to_string();
    let deploy_args_key_string = String::from_utf8_lossy(deploy_args_key.as_slice()).to_string();

    // the top level cbor map
    let mut cbor_buffer_map_pair: Vec<(ciborium::Value, ciborium::Value)> = Vec::new();

    // attrs cbor
    let mut attrs_buffer_vec = Vec::new();
    for byte in deploy_args.iter() {
        attrs_buffer_vec.push(ciborium::Value::Integer(Integer::from(*byte)));
    }
    cbor_buffer_map_pair.push((
        ciborium::Value::Text(deploy_args_key_string),
        ciborium::Value::Array(attrs_buffer_vec),
    ));

    // seed cbor
    let seed_string = String::from_utf8_lossy(seed.as_slice()).to_string();
    cbor_buffer_map_pair.push((
        ciborium::Value::Text(seed_key_string),
        ciborium::Value::Text(seed_string),
    ));

    // user_input cbor
    let user_input_string = String::from_utf8_lossy(user_input.as_slice()).to_string();
    cbor_buffer_map_pair.push((
        ciborium::Value::Text(user_input_key_string),
        ciborium::Value::Text(user_input_string),
    ));

    let mut top_buffer = Vec::new();
    ciborium::into_writer(&ciborium::Value::Map(cbor_buffer_map_pair), &mut top_buffer).expect("");

    let mut cost = gas_params.base;
    let total_length = user_input_key.len()
        + user_input.len()
        + seed_key.len()
        + seed.len()
        + deploy_args_key.len()
        + deploy_args.len();
    cost += gas_params.per_byte * NumBytes::new(total_length as u64);

    let ret = Value::vector_u8(top_buffer);
    Ok(NativeResult::ok(cost, smallvec![ret]))
}

pub fn make_all(
    gas_params: ArgsPackingGasParameters,
) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "native_pack_inscribe_generate_args",
        make_native(gas_params, native_pack_inscribe_generate_args),
    )];

    make_module_natives(natives)
}
