// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::{
    address::{Address, AddressType},
    secp256k1::Secp256k1,
    PublicKey, XOnlyPublicKey,
};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};
use moveos_stdlib::natives::helpers::{make_module_natives, make_native};
use moveos_types::state::MoveStructState;
use rooch_types::address::BitcoinAddress;
use smallvec::smallvec;
use std::{collections::VecDeque, str::FromStr};

pub const E_INVALID_ADDRESS: u64 = 1;

pub fn parse(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let addr_bytes = pop_arg!(args, VectorRef);
    let addr_ref = addr_bytes.as_bytes_ref();

    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(addr_ref.len() as u64);

    let Ok(addr_str) = std::str::from_utf8(&addr_ref) else {
        return Ok(NativeResult::err(cost, E_INVALID_ADDRESS));
    };

    let addr = match BitcoinAddress::from_str(addr_str) {
        Ok(addr) => addr,
        Err(_) => {
            return Ok(NativeResult::err(cost, E_INVALID_ADDRESS));
        }
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::struct_(addr.to_runtime_value_struct())],
    ))
}

/// Returns true if the given pubkey is directly related to the address payload.
pub fn verify_with_pk(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let pk_bytes = pop_arg!(args, VectorRef);
    let addr_bytes = pop_arg!(args, VectorRef);

    let pk_ref = pk_bytes.as_bytes_ref();
    let addr_ref = addr_bytes.as_bytes_ref();

    let cost = gas_params.base
        + gas_params.per_byte * NumBytes::new((pk_ref.len() + addr_ref.len()) as u64);

    let Ok(pk) = PublicKey::from_slice(&pk_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    let Ok(addr_str) = std::str::from_utf8(&addr_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    let addr = match Address::from_str(addr_str) {
        Ok(addr) => addr.assume_checked(),
        Err(_) => {
            return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
        }
    };

    let is_ok = match addr.address_type() {
        Some(AddressType::P2tr) => {
            let xonly_pubkey = XOnlyPublicKey::from(pk.inner);
            let secp = Secp256k1::verification_only();
            let trust_addr = Address::p2tr(&secp, xonly_pubkey, None, *addr.network());
            addr.is_related_to_pubkey(&pk) || trust_addr.to_string() == addr.to_string()
        }
        _ => addr.is_related_to_pubkey(&pk),
    };

    Ok(NativeResult::ok(cost, smallvec![Value::bool(is_ok)]))
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
    pub new: FromBytesGasParameters,
    pub verify_with_pk: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            new: FromBytesGasParameters::zeros(),
            verify_with_pk: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("parse", make_native(gas_params.new, parse)),
        (
            "verify_with_pk",
            make_native(gas_params.verify_with_pk, verify_with_pk),
        ),
    ];

    make_module_natives(natives)
}
