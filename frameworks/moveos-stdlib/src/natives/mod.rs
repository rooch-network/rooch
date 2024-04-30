// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};
use moveos_types::addresses::{MOVEOS_STD_ADDRESS, MOVE_STD_ADDRESS};

pub mod helpers;
pub mod moveos_stdlib;

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub move_stdlib: move_stdlib::natives::GasParameters,
    pub move_nursery: move_stdlib::natives::NurseryGasParameters,
    pub account: moveos_stdlib::account::GasParameters,
    pub type_info: moveos_stdlib::type_info::GasParameters,
    pub rlp: moveos_stdlib::rlp::GasParameters,
    pub bcd: moveos_stdlib::bcs::GasParameters,
    pub events: moveos_stdlib::event::GasParameters,
    pub test_helper: moveos_stdlib::test_helper::GasParameters,
    pub signer: moveos_stdlib::signer::GasParameters,
    pub move_module: moveos_stdlib::move_module::GasParameters,
    pub object: moveos_stdlib::object::GasParameters,
    pub json: moveos_stdlib::json::GasParameters,
    pub wasm: moveos_stdlib::wasm::GasParameters,
    pub tx_context: moveos_stdlib::tx_context::GasParameters,
    pub base58: moveos_stdlib::base58::GasParameters,
    pub bech32: moveos_stdlib::bech32::GasParameters,
    pub hash: moveos_stdlib::hash::GasParameters,
    pub bls12381: moveos_stdlib::bls12381::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: move_stdlib::natives::GasParameters::zeros(),
            move_nursery: move_stdlib::natives::NurseryGasParameters::zeros(),
            account: moveos_stdlib::account::GasParameters::zeros(),
            type_info: moveos_stdlib::type_info::GasParameters::zeros(),
            rlp: moveos_stdlib::rlp::GasParameters::zeros(),
            bcd: moveos_stdlib::bcs::GasParameters::zeros(),
            events: moveos_stdlib::event::GasParameters::zeros(),
            test_helper: moveos_stdlib::test_helper::GasParameters::zeros(),
            signer: moveos_stdlib::signer::GasParameters::zeros(),
            move_module: moveos_stdlib::move_module::GasParameters::zeros(),
            object: moveos_stdlib::object::GasParameters::zeros(),
            json: moveos_stdlib::json::GasParameters::zeros(),
            wasm: moveos_stdlib::wasm::GasParameters::zeros(),
            tx_context: moveos_stdlib::tx_context::GasParameters::zeros(),
            base58: moveos_stdlib::base58::GasParameters::zeros(),
            bech32: moveos_stdlib::bech32::GasParameters::zeros(),
            hash: moveos_stdlib::hash::GasParameters::zeros(),
            bls12381: moveos_stdlib::bls12381::GasParameters::zeros(),
        }
    }
}

/// A fixed base gas cost for a native function.
#[derive(Debug, Clone)]
pub struct BaseGasParameter {
    pub base: InternalGas,
}

impl BaseGasParameter {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::new(0),
        }
    }
}

pub fn all_natives(gas_params: GasParameters) -> NativeFunctionTable {
    let mut native_fun_table =
        move_stdlib::natives::all_natives(MOVE_STD_ADDRESS, gas_params.move_stdlib);

    // we only depend on the `debug` native function from the nursery
    let nursery_fun_table =
        move_stdlib::natives::nursery_natives(MOVE_STD_ADDRESS, gas_params.move_nursery);
    native_fun_table.extend(nursery_fun_table);

    let mut natives = vec![];

    macro_rules! add_natives {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    // moveos_stdlib natives
    add_natives!(
        "account",
        moveos_stdlib::account::make_all(gas_params.account)
    );
    add_natives!(
        "type_info",
        moveos_stdlib::type_info::make_all(gas_params.type_info)
    );
    add_natives!("rlp", moveos_stdlib::rlp::make_all(gas_params.rlp));
    add_natives!("bcs", moveos_stdlib::bcs::make_all(gas_params.bcd));
    add_natives!("event", moveos_stdlib::event::make_all(gas_params.events));
    add_natives!(
        "test_helper",
        moveos_stdlib::test_helper::make_all(gas_params.test_helper)
    );
    add_natives!("signer", moveos_stdlib::signer::make_all(gas_params.signer));
    add_natives!(
        "move_module",
        moveos_stdlib::move_module::make_all(gas_params.move_module)
    );
    add_natives!("object", moveos_stdlib::object::make_all(gas_params.object));
    add_natives!("json", moveos_stdlib::json::make_all(gas_params.json));
    add_natives!("wasm", moveos_stdlib::wasm::make_all(gas_params.wasm));
    add_natives!(
        "tx_context",
        moveos_stdlib::tx_context::make_all(gas_params.tx_context)
    );
    add_natives!("base58", moveos_stdlib::base58::make_all(gas_params.base58));
    add_natives!("bech32", moveos_stdlib::bech32::make_all(gas_params.bech32));
    add_natives!("hash", moveos_stdlib::hash::make_all(gas_params.hash));
    add_natives!(
        "bls12381",
        moveos_stdlib::bls12381::make_all(gas_params.bls12381)
    );

    let moveos_native_fun_table = make_table_from_iter(MOVEOS_STD_ADDRESS, natives);
    native_fun_table.extend(moveos_native_fun_table);

    native_fun_table
}
