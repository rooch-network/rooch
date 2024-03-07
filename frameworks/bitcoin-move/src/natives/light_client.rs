// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use better_any::Tid;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::InternalGas;
use move_core_types::resolver::ModuleResolver;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{StructRef, Value, Vector},
};
use moveos_stdlib::natives::helpers::{make_module_natives, make_native};
use moveos_types::state::{MoveState, MoveType};
use rooch_types::bitcoin::light_client::BitcoinLightClientModule;
use rooch_types::bitcoin::types::Witness;
use smallvec::smallvec;
use std::cmp::max;
use std::collections::VecDeque;
use tracing::error;

#[derive(Debug, Clone)]
pub struct GetBlockGasParameters {
    pub base: InternalGas,
}

impl GetBlockGasParameters {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

/// Rust implementation of get block
#[inline]
pub(crate) fn native_get_block_impl(
    // native fun native_get_block(block_hash: address): Block;
    gas_params: &GetBlockGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 0);
    debug_assert_eq!(args.len(), 1);

    let cost = gas_params.base;

    // TODO(gas): charge gas
    let block_hash_address = pop_arg!(args, AccountAddress);
    let witness_ref = pop_arg!(args, StructRef);
    let wintness_value = witness_ref.read_ref()?;
    let witness = Witness::from_runtime_value(wintness_value).map_err(|e| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
            .with_message(format!("Failed to parse witness: {}", e))
    })?;

    let bitcoin_witness = bitcoin::Witness::from_slice(witness.witness.as_slice());
    let inscriptions = get_block(&bitcoin_witness);
    let inscription_vm_type = context
        .load_type(&rooch_types::bitcoin::ord::InscriptionRecord::type_tag())
        .map_err(|e| e.to_partial())?;
    let val = Vector::pack(
        &inscription_vm_type,
        inscriptions
            .into_iter()
            .map(|i| {
                Into::<rooch_types::bitcoin::ord::InscriptionRecord>::into(i).to_runtime_value()
            })
            .collect::<Vec<_>>(),
    )?;

    Ok(NativeResult::ok(cost, smallvec![val]))
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub get_block: GetBlockGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            get_block: GetBlockGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "native_get_block",
        make_native(gas_params.get_block, native_get_block_impl),
    )];

    make_module_natives(natives)
}

pub fn get_block(witness: &bitcoin::Witness) -> Vec<Inscription> {
    witness
        .tapscript()
        .map(|script| match RawEnvelope::from_tapscript(script, 0usize) {
            Ok(envelopes) => envelopes
                .into_iter()
                .map(ParsedEnvelope::from)
                .map(|e| e.payload)
                .collect::<Vec<_>>(),
            Err(e) => {
                if tracing::enabled!(tracing::Level::TRACE) {
                    error!(
                        "Failed to parse tapscript: {}, witness:\n {:#?}",
                        e, witness
                    );
                }
                vec![]
            }
        })
        .unwrap_or_default()
}

pub fn from_transaction(transaction: &bitcoin::Transaction) -> Vec<Inscription> {
    transaction
        .input
        .iter()
        .flat_map(|tx_in| get_block(&tx_in.witness))
        .collect::<Vec<_>>()
}
