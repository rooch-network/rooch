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
    rlp: mos_stdlib::rlp::GasParameters,
    account: mos_framework::account::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: move_stdlib::natives::GasParameters::zeros(),
            rlp: mos_stdlib::rlp::GasParameters::zeros(),
            account: mos_framework::account::GasParameters::zeros(),
        }
    }
}

pub fn all_natives(gas_params: GasParameters) -> NativeFunctionTable {
    let mut native_fun_table =
        move_stdlib::natives::all_natives(*MOVE_STD_ADDRESS, gas_params.move_stdlib);

    let mut natives = vec![];

    macro_rules! add_natives {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }
    // mos_stdlib natives
    add_natives!("rlp", mos_stdlib::rlp::make_all(gas_params.rlp));

    // mos_framework natives
    add_natives!(
        "account",
        mos_framework::account::make_all(gas_params.account)
    );

    let mos_table = make_table_from_iter(*MOS_STD_ADDRESS, natives);
    native_fun_table.extend(mos_table);
    native_fun_table
}
