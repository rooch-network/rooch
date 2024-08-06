// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::{
    address::{Address, AddressType},
    hex::DisplayHex,
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
use musig2::{secp::Point, KeyAggContext};
use rooch_types::address::BitcoinAddress;
use smallvec::smallvec;
use std::{collections::VecDeque, str::FromStr};

pub const E_INVALID_ADDRESS: u64 = 1;
pub const E_ARG_NOT_VECTOR_U8: u64 = 2;
pub const E_INVALID_PUBLIC_KEY: u64 = 3;
pub const E_INVALID_THRESHOLD: u64 = 4;
pub const E_INVALID_KEY_EGG_CONTEXT: u64 = 5;

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

// optional function
/// Returns true if the given pubkey's bitcoin address is equal to the input bitcoin address.
pub fn verify_bitcoin_address_with_public_key(
    gas_params: &FromBytesGasParametersOptional,
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
    let cost = gas_params.base.unwrap()
        + gas_params.per_byte.unwrap()
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

// optional function
pub fn derive_multisig_pubkey_from_pubkeys(
    gas_params: &FromBytesGasParametersOptional,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let threshold_bytes = pop_arg!(args, u64);
    let pk_list = pop_arg!(args, Vec<Value>);

    let mut cost =
        gas_params.base.unwrap() + gas_params.per_byte.unwrap() * NumBytes::new(threshold_bytes);

    if pk_list.len() < threshold_bytes as usize {
        return Ok(NativeResult::err(cost, E_INVALID_THRESHOLD));
    }

    let mut pubkeys = Vec::new();
    for arg_value in pk_list.iter() {
        let value = arg_value.copy_value()?;
        match value.value_as::<Vec<u8>>() {
            Ok(v) => {
                match Point::from_slice(&v) {
                    Ok(pk_args) => {
                        cost += gas_params.per_byte.unwrap() * NumBytes::new(v.len() as u64);
                        pubkeys.push(pk_args);
                    }
                    Err(_) => {
                        return Ok(NativeResult::err(cost, E_INVALID_PUBLIC_KEY));
                    }
                };
            }
            Err(_) => {
                return Ok(NativeResult::err(cost, E_ARG_NOT_VECTOR_U8));
            }
        }
    }

    let key_agg_ctx = match KeyAggContext::new(pubkeys) {
        Ok(key_agg_ctx) => key_agg_ctx,
        Err(_) => {
            return Ok(NativeResult::err(cost, E_INVALID_KEY_EGG_CONTEXT));
        }
    };

    let aggregated_pubkey: Point = key_agg_ctx.aggregated_pubkey();

    let pubkey = aggregated_pubkey.serialize();

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(pubkey)]))
}

// optional function
pub fn derive_bitcoin_taproot_address_from_pubkey(
    gas_params: &FromBytesGasParametersOptional,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let pubkey_bytes = pop_arg!(args, VectorRef);

    let pubkey_ref = pubkey_bytes.as_bytes_ref();

    let cost = gas_params.base.unwrap()
        + gas_params.per_byte.unwrap() * NumBytes::new(pubkey_ref.len() as u64);
    let bitcoin_pubkey = match bitcoin::PublicKey::from_slice(&pubkey_ref) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            tracing::debug!(
                "Failed to parse public key:{:?}, error: {:?}",
                pubkey_ref.as_hex(),
                e
            );
            return Ok(NativeResult::err(cost, E_INVALID_PUBLIC_KEY));
        }
    };
    let internal_key = XOnlyPublicKey::from(bitcoin_pubkey);

    let secp = bitcoin::secp256k1::Secp256k1::verification_only();
    let bitcoin_addr = BitcoinAddress::from(bitcoin::Address::p2tr(
        &secp,
        internal_key,
        None,
        bitcoin::Network::Bitcoin,
    ));

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::struct_(bitcoin_addr.to_runtime_value_struct())],
    ))
}

// optional params
#[derive(Debug, Clone)]
pub struct FromBytesGasParametersOptional {
    pub base: Option<InternalGas>,
    pub per_byte: Option<InternalGasPerByte>,
}

impl FromBytesGasParametersOptional {
    pub fn zeros() -> Self {
        Self {
            base: None,
            per_byte: None,
        }
    }
}

impl FromBytesGasParametersOptional {
    pub fn is_empty(&self) -> bool {
        self.base.is_none() || self.per_byte.is_none()
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub new: FromBytesGasParameters,
    pub verify_with_pk: FromBytesGasParameters,
    pub verify_bitcoin_address_with_public_key: FromBytesGasParametersOptional,
    pub derive_multisig_pubkey_from_pubkeys: FromBytesGasParametersOptional,
    pub derive_bitcoin_taproot_address_from_pubkey: FromBytesGasParametersOptional,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            new: FromBytesGasParameters::zeros(),
            verify_with_pk: FromBytesGasParameters::zeros(),
            verify_bitcoin_address_with_public_key: FromBytesGasParametersOptional::zeros(),
            derive_multisig_pubkey_from_pubkeys: FromBytesGasParametersOptional::zeros(),
            derive_bitcoin_taproot_address_from_pubkey: FromBytesGasParametersOptional::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = [
        ("parse", make_native(gas_params.new, parse)),
        (
            "verify_with_pk",
            make_native(gas_params.verify_with_pk, verify_with_pk),
        ),
    ]
    .to_vec();

    if !gas_params.verify_bitcoin_address_with_public_key.is_empty() {
        natives.push((
            "verify_bitcoin_address_with_public_key",
            make_native(
                gas_params.verify_bitcoin_address_with_public_key,
                verify_bitcoin_address_with_public_key,
            ),
        ));
    }

    if !gas_params.derive_multisig_pubkey_from_pubkeys.is_empty() {
        natives.push((
            "derive_multisig_pubkey_from_pubkeys",
            make_native(
                gas_params.derive_multisig_pubkey_from_pubkeys,
                derive_multisig_pubkey_from_pubkeys,
            ),
        ));
    }

    if !gas_params
        .derive_bitcoin_taproot_address_from_pubkey
        .is_empty()
    {
        natives.push((
            "derive_bitcoin_taproot_address_from_pubkey",
            make_native(
                gas_params.derive_bitcoin_taproot_address_from_pubkey,
                derive_bitcoin_taproot_address_from_pubkey,
            ),
        ));
    }

    make_module_natives(natives)
}
