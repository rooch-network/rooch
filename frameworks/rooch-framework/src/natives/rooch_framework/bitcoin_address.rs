// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::{
    address::{Address, AddressType},
    secp256k1::Secp256k1,
    PublicKey, XOnlyPublicKey,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{StructRef, Value, VectorRef},
};
use moveos_stdlib::natives::helpers::{make_module_natives, make_native};
use moveos_types::state::{MoveState, MoveStructState};
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

/// Returns true if the given pubkey's bitcoin address is equal to the input bitcoin address.
pub fn verify_bitcoin_address_with_public_key(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let pk_bytes = pop_arg!(args, VectorRef);
    let addr_bytes = pop_arg!(args, StructRef);

    let pk_ref = pk_bytes.as_bytes_ref();
    let addr_value = addr_bytes.read_ref()?;

    let bitcoin_addr = BitcoinAddress::from_runtime_value(addr_value).map_err(|e| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
            .with_message(format!("Failed to parse bitcoin address: {}", e))
    })?;
    let cost = gas_params.base
        + gas_params.per_byte
            * NumBytes::new((pk_ref.len() + bitcoin_addr.to_bytes().len()) as u64);

    // TODO: convert to internal rooch public key and to bitcoin address?
    let Ok(pk) = PublicKey::from_slice(&pk_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    // TODO: compare the input bitcoin address with the converted bitcoin address
    let addr = match Address::from_str(&bitcoin_addr.to_string()) {
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

// TODO: derive_multi_sign_address
pub fn derive_multi_sign_address(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    todo!();
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
    pub parse: FromBytesGasParameters,
    pub verify_bitcoin_address_with_public_key: FromBytesGasParameters,
    pub derive_multi_sign_address: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            parse: FromBytesGasParameters::zeros(),
            verify_bitcoin_address_with_public_key: FromBytesGasParameters::zeros(),
            derive_multi_sign_address: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("parse", make_native(gas_params.parse, parse)),
        (
            "verify_bitcoin_address_with_public_key",
            make_native(
                gas_params.verify_bitcoin_address_with_public_key,
                verify_bitcoin_address_with_public_key,
            ),
        ),
        (
            "derive_multi_sign_address",
            make_native(
                gas_params.derive_multi_sign_address,
                derive_multi_sign_address,
            ),
        ),
    ];

    make_module_natives(natives)
}
