// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_vm_runtime::native_functions::make_table_from_iter;
use move_vm_runtime::native_functions::NativeFunctionTable;
use rooch_framework::natives::gas_parameter::gas_member::InitialGasSchedule;
use rooch_framework::natives::gas_parameter::gas_member::{
    FromOnChainGasSchedule, ToOnChainGasSchedule,
};
use rooch_types::addresses::BITCOIN_MOVE_ADDRESS;
use std::collections::BTreeMap;

pub mod gas_parameter;
pub mod light_client;
pub mod ord;

#[derive(Debug, Clone)]
pub struct GasParameters {
    ord: ord::GasParameters,
    light_client: light_client::GasParameters,
}

impl FromOnChainGasSchedule for GasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            ord: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            light_client: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
        })
    }
}

impl ToOnChainGasSchedule for GasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entires = self.ord.to_on_chain_gas_schedule();
        entires.extend(self.light_client.to_on_chain_gas_schedule());
        entires
    }
}

impl InitialGasSchedule for GasParameters {
    fn initial() -> Self {
        Self {
            ord: InitialGasSchedule::initial(),
            light_client: InitialGasSchedule::initial(),
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
            ord: ord::GasParameters::zeros(),
            light_client: light_client::GasParameters::zeros(),
        }
    }
}

pub fn all_natives(gas_params: GasParameters) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    add_natives!("ord", ord::make_all(gas_params.ord));
    add_natives!(
        "light_client",
        light_client::make_all(gas_params.light_client)
    );

    make_table_from_iter(BITCOIN_MOVE_ADDRESS, natives)
}
