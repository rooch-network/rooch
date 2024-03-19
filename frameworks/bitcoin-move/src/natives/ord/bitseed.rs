use std::collections::VecDeque;

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use serde_json::Number;
use serde_json::Value as JSONValue;
use smallvec::smallvec;

use moveos_stdlib::natives::moveos_stdlib::wasm::E_CBOR_MARSHAL_FAILED;

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
pub(crate) fn native_pack_inscribe_generate_args(
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
    let mut buffer_map = serde_json::Map::new();

    // attrs
    let mut attrs_buffer_vec = Vec::new();
    for byte in deploy_args.iter() {
        attrs_buffer_vec.push(serde_json::Value::Number(Number::from(byte.clone())));
    }

    buffer_map.insert(
        deploy_args_key_string,
        serde_json::Value::Array(attrs_buffer_vec),
    );

    // seed
    let seed_string = String::from_utf8_lossy(seed.as_slice()).to_string();
    buffer_map.insert(seed_key_string, serde_json::Value::String(seed_string));

    // user_input
    let user_input_string = String::from_utf8_lossy(user_input.as_slice()).to_string();
    buffer_map.insert(
        user_input_key_string,
        serde_json::Value::String(user_input_string),
    );

    // marshal the cbor map to bytes
    let top_buffer_map = JSONValue::Object(buffer_map);
    let mut top_buffer = Vec::new();
    match ciborium::into_writer(&top_buffer_map, &mut top_buffer) {
        Ok(_) => {}
        Err(_) => {
            return Ok(NativeResult::err(gas_params.base, E_CBOR_MARSHAL_FAILED));
        }
    }

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
