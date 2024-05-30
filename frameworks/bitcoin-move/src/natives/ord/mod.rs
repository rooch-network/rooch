// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
pub mod envelope;
#[allow(dead_code)]
pub mod inscription;
#[allow(dead_code)]
pub mod inscription_id;
pub mod media;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test;

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
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
use rooch_types::bitcoin::types::Witness;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::collections::VecDeque;
use tracing::error;
use {envelope::ParsedEnvelope, envelope::RawEnvelope, inscription::Inscription};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct FromWitnessGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl FromWitnessGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/// Rust implementation of parse Inscription from witness
#[inline]
pub(crate) fn native_from_witness(
    gas_params: &FromWitnessGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 0);
    debug_assert_eq!(args.len(), 1);

    let mut cost = gas_params.base;

    let witness_ref = pop_arg!(args, StructRef);
    let wintness_value = witness_ref.read_ref()?;
    let witness = Witness::from_runtime_value(wintness_value).map_err(|e| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
            .with_message(format!("Failed to parse witness: {}", e))
    })?;
    cost += gas_params.per_byte
        * NumBytes::new(
            witness
                .witness
                .iter()
                .map(|inner_vec| inner_vec.len())
                .sum::<usize>() as u64,
        );
    let bitcoin_witness = bitcoin::Witness::from_slice(witness.witness.as_slice());
    let inscriptions = from_witness(&bitcoin_witness);
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

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct GasParameters {
    pub from_witness: FromWitnessGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            from_witness: FromWitnessGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "from_witness",
        make_native(gas_params.from_witness, native_from_witness),
    )];

    make_module_natives(natives)
}

pub fn from_witness(witness: &bitcoin::Witness) -> Vec<Inscription> {
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
        .flat_map(|tx_in| from_witness(&tx_in.witness))
        .collect::<Vec<_>>()
}
