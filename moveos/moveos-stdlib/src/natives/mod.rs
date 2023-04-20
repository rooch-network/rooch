// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::{MOVEOS_STD_ADDRESS, MOVE_STD_ADDRESS},
    natives::moveos_stdlib::any_table,
};
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};

mod helpers;
pub mod moveos_stdlib;
pub mod rooch_framework;

#[derive(Debug, Clone)]
pub struct GasParameters {
    move_stdlib: move_stdlib::natives::GasParameters,
    move_nursery: move_stdlib::natives::NurseryGasParameters,
    table_extension: move_table_extension::GasParameters,
    type_info: moveos_stdlib::type_info::GasParameters,
    rlp: moveos_stdlib::rlp::GasParameters,
    account: rooch_framework::account::GasParameters,
    bcd: moveos_stdlib::bcd::GasParameters,
    tx_context: moveos_stdlib::tx_context::GasParameters,
    object: moveos_stdlib::object::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: move_stdlib::natives::GasParameters::zeros(),
            move_nursery: move_stdlib::natives::NurseryGasParameters::zeros(),
            table_extension: move_table_extension::GasParameters::zeros(),
            type_info: moveos_stdlib::type_info::GasParameters::zeros(),
            rlp: moveos_stdlib::rlp::GasParameters::zeros(),
            account: rooch_framework::account::GasParameters::zeros(),
            bcd: moveos_stdlib::bcd::GasParameters::zeros(),
            tx_context: moveos_stdlib::tx_context::GasParameters::zeros(),
            object: moveos_stdlib::object::GasParameters::zeros(),
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
        move_stdlib::natives::all_natives(*MOVE_STD_ADDRESS, gas_params.move_stdlib);

    let nursery_fun_table =
        move_stdlib::natives::nursery_natives(*MOVE_STD_ADDRESS, gas_params.move_nursery);
    native_fun_table.extend(nursery_fun_table);

    let table_fun_table = move_table_extension::table_natives(
        *MOVEOS_STD_ADDRESS,
        gas_params.table_extension.clone(),
    );
    native_fun_table.extend(table_fun_table);

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
        "type_info",
        moveos_stdlib::type_info::make_all(gas_params.type_info)
    );
    add_natives!("rlp", moveos_stdlib::rlp::make_all(gas_params.rlp));
    add_natives!("bcd", moveos_stdlib::bcd::make_all(gas_params.bcd));
    add_natives!(
        "tx_context",
        moveos_stdlib::tx_context::make_all(gas_params.tx_context)
    );
    add_natives!("object", moveos_stdlib::object::make_all(gas_params.object));

    // rooch_framework natives
    add_natives!(
        "account",
        rooch_framework::account::make_all(gas_params.account)
    );

    let moveos_native_fun_table = make_table_from_iter(*MOVEOS_STD_ADDRESS, natives);
    native_fun_table.extend(moveos_native_fun_table);

    let any_table_fun_table =
        any_table::table_natives(*MOVEOS_STD_ADDRESS, gas_params.table_extension);
    native_fun_table.extend(any_table_fun_table);

    native_fun_table
}
