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

mod gas_parameter;
pub mod ord;

#[derive(Debug, Clone)]
pub struct GasParameters {
    ord: ord::GasParameters,
}

impl FromOnChainGasSchedule for GasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            ord: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
        })
    }
}

impl ToOnChainGasSchedule for GasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        self.ord.to_on_chain_gas_schedule()
    }
}

impl InitialGasSchedule for GasParameters {
    fn initial() -> Self {
        Self {
            ord: InitialGasSchedule::initial(),
        }
    }
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            ord: ord::GasParameters::zeros(),
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

    make_table_from_iter(BITCOIN_MOVE_ADDRESS, natives)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_parameters() {
        let gas_parameters = GasParameters::initial();
        let on_chain_gas_schedule = gas_parameters.to_on_chain_gas_schedule();
        let entries = on_chain_gas_schedule
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<BTreeMap<String, u64>>();
        let gas_parameters_from_on_chain =
            GasParameters::from_on_chain_gas_schedule(&entries).unwrap();
        assert_eq!(
            gas_parameters.to_on_chain_gas_schedule(),
            gas_parameters_from_on_chain.to_on_chain_gas_schedule()
        );
    }
}
