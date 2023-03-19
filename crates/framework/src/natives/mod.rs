// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::{MOS_STD_ADDRESS, MOVE_STD_ADDRESS};
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};

mod helpers;
pub mod mos_framework;
pub mod mos_stdlib;

#[derive(Debug, Clone)]
pub struct GasParameters {
    move_stdlib: move_stdlib::natives::GasParameters,
    move_nursery: move_stdlib::natives::NurseryGasParameters,
    table_extension: move_table_extension::GasParameters,
    type_info: mos_stdlib::type_info::GasParameters,
    rlp: mos_stdlib::rlp::GasParameters,
    account: mos_framework::account::GasParameters,
    bcd: mos_stdlib::bcd::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: move_stdlib::natives::GasParameters::zeros(),
            move_nursery: move_stdlib::natives::NurseryGasParameters::zeros(),
            table_extension: move_table_extension::GasParameters::zeros(),
            type_info: mos_stdlib::type_info::GasParameters::zeros(),
            rlp: mos_stdlib::rlp::GasParameters::zeros(),
            account: mos_framework::account::GasParameters::zeros(),
            bcd: mos_stdlib::bcd::GasParameters::zeros(),
        }
    }
}

pub fn all_natives(gas_params: GasParameters) -> NativeFunctionTable {
    let mut native_fun_table =
        move_stdlib::natives::all_natives(*MOVE_STD_ADDRESS, gas_params.move_stdlib);

    let nursery_fun_table =
        move_stdlib::natives::nursery_natives(*MOVE_STD_ADDRESS, gas_params.move_nursery);
    native_fun_table.extend(nursery_fun_table);

    let table_fun_table =
        move_table_extension::table_natives(*MOS_STD_ADDRESS, gas_params.table_extension);
    native_fun_table.extend(table_fun_table);

    let mut natives = vec![];

    macro_rules! add_natives {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    // mos_stdlib natives
    add_natives!(
        "type_info",
        mos_stdlib::type_info::make_all(gas_params.type_info)
    );
    add_natives!("rlp", mos_stdlib::rlp::make_all(gas_params.rlp));
    add_natives!("bcd", mos_stdlib::bcd::make_all(gas_params.bcd));

    // mos_framework natives
    add_natives!(
        "account",
        mos_framework::account::make_all(gas_params.account)
    );

    let mos_table = make_table_from_iter(*MOS_STD_ADDRESS, natives);
    native_fun_table.extend(mos_table);

    native_fun_table
}
