// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::gas_member::{FromOnChainGasSchedule, InitialGasSchedule};
use crate::ROOCH_FRAMEWORK_ADDRESS;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};
use moveos_stdlib::natives::GasParameters as MoveOSGasParameters;
use std::collections::BTreeMap;

pub mod helpers {
    pub use moveos_stdlib::natives::helpers::*;
}
pub mod gas_parameter;
pub mod rooch_framework;

#[derive(Debug, Clone)]
pub struct GasParameters {
    moveos_stdlib: moveos_stdlib::natives::GasParameters,
    account: rooch_framework::account::GasParameters,
    hash: rooch_framework::crypto::hash::GasParameters,
    ed25519: rooch_framework::crypto::ed25519::GasParameters,
    ecdsa_k1: rooch_framework::crypto::ecdsa_k1::GasParameters,
    ecdsa_k1_recoverable: rooch_framework::crypto::ecdsa_k1_recoverable::GasParameters,
    schnorr: rooch_framework::crypto::schnorr::GasParameters,
    encoding: rooch_framework::crypto::encoding::GasParameters,
    decoding: rooch_framework::crypto::decoding::GasParameters,
    bcs: rooch_framework::bcs::GasParameters,
    ord: rooch_framework::bitcoin::ord::GasParameters,
}

impl FromOnChainGasSchedule for GasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            moveos_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap(),
            account: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            hash: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            ed25519: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            ecdsa_k1: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            ecdsa_k1_recoverable: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap(),
            schnorr: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            encoding: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            decoding: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            bcs: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            ord: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
        })
    }
}

impl InitialGasSchedule for GasParameters {
    fn initial() -> Self {
        Self {
            moveos_stdlib: InitialGasSchedule::initial(),
            account: InitialGasSchedule::initial(),
            hash: InitialGasSchedule::initial(),
            ed25519: InitialGasSchedule::initial(),
            ecdsa_k1: InitialGasSchedule::initial(),
            ecdsa_k1_recoverable: InitialGasSchedule::initial(),
            schnorr: InitialGasSchedule::initial(),
            encoding: InitialGasSchedule::initial(),
            decoding: InitialGasSchedule::initial(),
            bcs: InitialGasSchedule::initial(),
            ord: InitialGasSchedule::initial(),
        }
    }
}

impl FromOnChainGasSchedule for MoveOSGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            move_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            move_nursery: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            table_extension: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap(),
            type_info: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            rlp: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            bcd: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            events: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            test_helper: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            signer: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            move_module: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            object: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            json: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
        })
    }
}

impl InitialGasSchedule for MoveOSGasParameters {
    fn initial() -> Self {
        Self {
            move_stdlib: InitialGasSchedule::initial(),
            move_nursery: InitialGasSchedule::initial(),
            table_extension: InitialGasSchedule::initial(),
            type_info: InitialGasSchedule::initial(),
            rlp: InitialGasSchedule::initial(),
            bcd: InitialGasSchedule::initial(),
            events: InitialGasSchedule::initial(),
            test_helper: InitialGasSchedule::initial(),
            signer: InitialGasSchedule::initial(),
            move_module: InitialGasSchedule::initial(),
            object: InitialGasSchedule::initial(),
            json: InitialGasSchedule::initial(),
        }
    }
}

pub fn get_global_gas_parameter() {
    let gas_parameter = GasParameters::initial();
    println!("global gas parameter {:?}", gas_parameter);
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            moveos_stdlib: moveos_stdlib::natives::GasParameters::zeros(),
            account: rooch_framework::account::GasParameters::zeros(),
            hash: rooch_framework::crypto::hash::GasParameters::zeros(),
            ed25519: rooch_framework::crypto::ed25519::GasParameters::zeros(),
            ecdsa_k1: rooch_framework::crypto::ecdsa_k1::GasParameters::zeros(),
            ecdsa_k1_recoverable:
                rooch_framework::crypto::ecdsa_k1_recoverable::GasParameters::zeros(),
            schnorr: rooch_framework::crypto::schnorr::GasParameters::zeros(),
            encoding: rooch_framework::crypto::encoding::GasParameters::zeros(),
            decoding: rooch_framework::crypto::decoding::GasParameters::zeros(),
            bcs: rooch_framework::bcs::GasParameters::zeros(),
            ord: rooch_framework::bitcoin::ord::GasParameters::zeros(),
        }
    }
}

pub fn all_natives(gas_params: GasParameters) -> NativeFunctionTable {
    let mut native_fun_table = moveos_stdlib::natives::all_natives(gas_params.moveos_stdlib);

    let mut natives = vec![];

    macro_rules! add_natives {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    // rooch_framework natives
    add_natives!(
        "account",
        rooch_framework::account::make_all(gas_params.account)
    );
    add_natives!(
        "hash",
        rooch_framework::crypto::hash::make_all(gas_params.hash)
    );
    add_natives!(
        "ed25519",
        rooch_framework::crypto::ed25519::make_all(gas_params.ed25519)
    );
    add_natives!(
        "ecdsa_k1",
        rooch_framework::crypto::ecdsa_k1::make_all(gas_params.ecdsa_k1)
    );
    add_natives!(
        "ecdsa_k1_recoverable",
        rooch_framework::crypto::ecdsa_k1_recoverable::make_all(gas_params.ecdsa_k1_recoverable)
    );
    add_natives!(
        "schnorr",
        rooch_framework::crypto::schnorr::make_all(gas_params.schnorr)
    );
    add_natives!(
        "encoding",
        rooch_framework::crypto::encoding::make_all(gas_params.encoding)
    );
    add_natives!(
        "decoding",
        rooch_framework::crypto::decoding::make_all(gas_params.decoding)
    );
    add_natives!("bcs", rooch_framework::bcs::make_all(gas_params.bcs));
    add_natives!(
        "ord",
        rooch_framework::bitcoin::ord::make_all(gas_params.ord)
    );

    let rooch_native_fun_table = make_table_from_iter(ROOCH_FRAMEWORK_ADDRESS, natives);
    native_fun_table.extend(rooch_native_fun_table);

    native_fun_table
}
