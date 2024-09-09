// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};
use rooch_framework::natives::gas_parameter::gas_member::{
    FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule,
};
use rooch_types::addresses::ROOCH_NURSERY_ADDRESS;
use std::collections::BTreeMap;

pub mod helper;
pub mod gas_parameter;
pub mod wasm;
pub mod cosmwasm_vm; 

#[derive(Debug, Clone)]
pub struct GasParameters {
    wasm: crate::natives::wasm::GasParameters,
    cosmwasm_vm: crate::natives::cosmwasm_vm::GasParameters, // Add this field
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            wasm: crate::natives::wasm::GasParameters::zeros(),
            cosmwasm_vm: crate::natives::cosmwasm_vm::GasParameters::zeros(), 
        }
    }
}

impl FromOnChainGasSchedule for GasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            wasm: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap_or_else(crate::natives::wasm::GasParameters::zeros),
            cosmwasm_vm: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap_or_else(crate::natives::cosmwasm_vm::GasParameters::zeros), 
        })
    }
}

impl ToOnChainGasSchedule for GasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut gas_schedule = self.wasm.to_on_chain_gas_schedule();
        gas_schedule.extend(self.cosmwasm_vm.to_on_chain_gas_schedule()); 
        gas_schedule
    }
}

impl InitialGasSchedule for GasParameters {
    fn initial() -> Self {
        Self {
            wasm: InitialGasSchedule::initial(),
            cosmwasm_vm: InitialGasSchedule::initial(), 
        }
    }
}

pub fn all_natives(gas_params: GasParameters) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives {
        ($module_name:expr, $natives:expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }
    add_natives!("wasm", wasm::make_all(gas_params.wasm));
    add_natives!("cosmwasm_vm", cosmwasm_vm::make_all(gas_params.cosmwasm_vm)); 

    let rooch_nursery_native_fun_table = make_table_from_iter(ROOCH_NURSERY_ADDRESS, natives);

    rooch_nursery_native_fun_table
}
