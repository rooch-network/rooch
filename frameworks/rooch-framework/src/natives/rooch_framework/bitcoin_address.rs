// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::{
    address::{Address, AddressType},
    hashes::Hash,
    hex::DisplayHex,
    secp256k1::Secp256k1,
    PublicKey, TapNodeHash, XOnlyPublicKey,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress,
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
use moveos_types::{
    move_std::option::MoveOption,
    state::{MoveState, MoveStructState},
};
use rooch_types::address::BitcoinAddress;
use smallvec::smallvec;
use std::{collections::VecDeque, str::FromStr};

pub const E_INVALID_ADDRESS: u64 = 1;
pub const E_ARG_NOT_VECTOR_U8: u64 = 2;
pub const E_INVALID_PUBLIC_KEY: u64 = 3;
pub const E_INVALID_THRESHOLD: u64 = 4;
pub const E_INVALID_KEY_EGG_CONTEXT: u64 = 5;
pub const E_DEPRECATED: u64 = 6;

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

pub fn derive_bitcoin_taproot_address(
    gas_params: &FromBytesGasParametersOptional,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(args.len(), 2);
    let merkle_root_arg = args.pop_back().expect("merkle root is missing");
    let internal_pubkey = pop_arg!(args, VectorRef);
    let merkle_root: Option<AccountAddress> = MoveOption::from_runtime_value(merkle_root_arg)
        .map_err(|e| {
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message(format!("Failed to parse merkle root: {}", e))
        })?
        .into();

    let internal_pubkey_ref = internal_pubkey.as_bytes_ref();

    let gas_base = gas_params.base.expect("base gas is missing");
    let gas_per_byte = gas_params.per_byte.expect("per byte gas is missing");

    let merkle_root_bytes_len = match &merkle_root {
        Some(_addr) => AccountAddress::LENGTH,
        None => 1,
    };
    let cost = gas_base
        + gas_per_byte * NumBytes::new(internal_pubkey_ref.len() as u64)
        + gas_per_byte * NumBytes::new(merkle_root_bytes_len as u64);

    let internal_key = match to_x_only_public_key(&internal_pubkey_ref) {
        Ok(internal_key) => internal_key,
        Err(e) => {
            tracing::debug!(
                "Failed to parse public key:{:?}, error: {:?}",
                internal_pubkey_ref.as_hex(),
                e
            );
            return Ok(NativeResult::err(cost, E_INVALID_PUBLIC_KEY));
        }
    };
    let merkle_root = merkle_root.map(|addr| {
        TapNodeHash::from_slice(addr.as_slice()).expect("address to merkle root should success")
    });
    let secp = bitcoin::secp256k1::Secp256k1::verification_only();
    let bitcoin_addr = BitcoinAddress::from(bitcoin::Address::p2tr(
        &secp,
        internal_key,
        merkle_root,
        bitcoin::Network::Bitcoin,
    ));

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::struct_(bitcoin_addr.to_runtime_value_struct())],
    ))
}

fn to_x_only_public_key(bytes: &[u8]) -> Result<XOnlyPublicKey, PartialVMError> {
    match bytes.len() {
        32 => XOnlyPublicKey::from_slice(bytes).map_err(|e| {
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message(format!("Failed to parse public key: {}", e))
        }),
        _ => {
            let public_key = PublicKey::from_slice(bytes).map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("Failed to parse public key: {}", e))
            })?;
            Ok(XOnlyPublicKey::from(public_key))
        }
    }
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
    pub parse: FromBytesGasParameters,
    pub verify_bitcoin_address_with_public_key: FromBytesGasParametersOptional,
    pub derive_bitcoin_taproot_address: FromBytesGasParametersOptional,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            parse: FromBytesGasParameters::zeros(),
            verify_bitcoin_address_with_public_key: FromBytesGasParametersOptional::zeros(),
            derive_bitcoin_taproot_address: FromBytesGasParametersOptional::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = [("parse", make_native(gas_params.parse, parse))].to_vec();

    if !gas_params.verify_bitcoin_address_with_public_key.is_empty() {
        natives.push((
            "verify_bitcoin_address_with_public_key",
            make_native(
                gas_params.verify_bitcoin_address_with_public_key,
                verify_bitcoin_address_with_public_key,
            ),
        ));
    }

    if !gas_params.derive_bitcoin_taproot_address.is_empty() {
        natives.push((
            "derive_bitcoin_taproot_address",
            make_native(
                gas_params.derive_bitcoin_taproot_address,
                derive_bitcoin_taproot_address,
            ),
        ));
    }

    make_module_natives(natives)
}
