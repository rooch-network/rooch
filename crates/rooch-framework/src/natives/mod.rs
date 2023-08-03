// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::ROOCH_FRAMEWORK_ADDRESS;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};

pub mod helpers {
    pub use moveos_stdlib::natives::helpers::*;
}
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

    let rooch_native_fun_table = make_table_from_iter(ROOCH_FRAMEWORK_ADDRESS, natives);
    native_fun_table.extend(rooch_native_fun_table);

    native_fun_table
}
