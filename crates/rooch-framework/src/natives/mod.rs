// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::gas_member::{
    FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule,
};
use crate::ROOCH_FRAMEWORK_ADDRESS;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};
use moveos::gas::table::{
    from_on_chain_gas_schedule_to_instruction_parameter,
    from_on_chain_gas_schedule_to_misc_parameter, from_on_chain_gas_schedule_to_storage_parameter,
    initial_instruction_parameter, initial_misc_parameter, initial_storage_parameter,
    instruction_parameter_to_on_chain_gas_schedule, misc_parameter_to_on_chain_gas_schedule,
    storage_parameter_to_on_chain_gas_schedule, AbstractValueSizeGasParameter,
    InstructionParameter, StorageGasParameter,
};
use moveos_stdlib::natives::GasParameters as MoveOSStdlibGasParameters;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod helpers {
    pub use moveos_stdlib::natives::helpers::*;
}
pub mod gas_parameter;
pub mod rooch_framework;

#[derive(Debug, Clone)]
pub struct NativeGasParameters {
    moveos_stdlib: moveos_stdlib::natives::GasParameters,
    hash: rooch_framework::crypto::hash::GasParameters,
    ed25519: rooch_framework::crypto::ed25519::GasParameters,
    ecdsa_k1: rooch_framework::crypto::ecdsa_k1::GasParameters,
    encoding: rooch_framework::crypto::encoding::GasParameters,
    decoding: rooch_framework::crypto::decoding::GasParameters,
    bcs: rooch_framework::bcs::GasParameters,
}

impl FromOnChainGasSchedule for NativeGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            moveos_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap(),
            hash: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            ed25519: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            ecdsa_k1: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            encoding: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            decoding: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            bcs: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
        })
    }
}

impl ToOnChainGasSchedule for NativeGasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entires = self.moveos_stdlib.to_on_chain_gas_schedule();
        entires.extend(self.hash.to_on_chain_gas_schedule());
        entires.extend(self.ed25519.to_on_chain_gas_schedule());
        entires.extend(self.ecdsa_k1.to_on_chain_gas_schedule());
        entires.extend(self.encoding.to_on_chain_gas_schedule());
        entires.extend(self.decoding.to_on_chain_gas_schedule());
        entires.extend(self.bcs.to_on_chain_gas_schedule());
        entires
    }
}

impl InitialGasSchedule for NativeGasParameters {
    fn initial() -> Self {
        Self {
            moveos_stdlib: InitialGasSchedule::initial(),
            hash: InitialGasSchedule::initial(),
            ed25519: InitialGasSchedule::initial(),
            ecdsa_k1: InitialGasSchedule::initial(),
            encoding: InitialGasSchedule::initial(),
            decoding: InitialGasSchedule::initial(),
            bcs: InitialGasSchedule::initial(),
        }
    }
}

impl FromOnChainGasSchedule for MoveOSStdlibGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            move_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            move_nursery: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            account: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            type_info: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            rlp: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            bcd: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            events: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            test_helper: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            signer: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            move_module: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            object: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            json: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            wasm: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
            tx_context: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule).unwrap(),
        })
    }
}

impl ToOnChainGasSchedule for MoveOSStdlibGasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entires = self.move_stdlib.to_on_chain_gas_schedule();
        entires.extend(self.move_nursery.to_on_chain_gas_schedule());
        entires.extend(self.account.to_on_chain_gas_schedule());
        entires.extend(self.type_info.to_on_chain_gas_schedule());
        entires.extend(self.rlp.to_on_chain_gas_schedule());
        entires.extend(self.bcd.to_on_chain_gas_schedule());
        entires.extend(self.bcd.to_on_chain_gas_schedule());
        entires.extend(self.events.to_on_chain_gas_schedule());
        entires.extend(self.test_helper.to_on_chain_gas_schedule());
        entires.extend(self.signer.to_on_chain_gas_schedule());
        entires.extend(self.move_module.to_on_chain_gas_schedule());
        entires.extend(self.object.to_on_chain_gas_schedule());
        entires.extend(self.json.to_on_chain_gas_schedule());
        entires.extend(self.wasm.to_on_chain_gas_schedule());
        entires.extend(self.tx_context.to_on_chain_gas_schedule());
        entires
    }
}

impl InitialGasSchedule for MoveOSStdlibGasParameters {
    fn initial() -> Self {
        Self {
            move_stdlib: InitialGasSchedule::initial(),
            move_nursery: InitialGasSchedule::initial(),
            account: InitialGasSchedule::initial(),
            type_info: InitialGasSchedule::initial(),
            rlp: InitialGasSchedule::initial(),
            bcd: InitialGasSchedule::initial(),
            events: InitialGasSchedule::initial(),
            test_helper: InitialGasSchedule::initial(),
            signer: InitialGasSchedule::initial(),
            move_module: InitialGasSchedule::initial(),
            object: InitialGasSchedule::initial(),
            json: InitialGasSchedule::initial(),
            wasm: InitialGasSchedule::initial(),
            tx_context: InitialGasSchedule::initial(),
        }
    }
}

pub fn get_global_gas_parameter() {
    let gas_parameter = NativeGasParameters::initial();
    println!("global gas parameter {:?}", gas_parameter);
}

impl NativeGasParameters {
    pub fn zeros() -> Self {
        Self {
            moveos_stdlib: moveos_stdlib::natives::GasParameters::zeros(),
            hash: rooch_framework::crypto::hash::GasParameters::zeros(),
            ed25519: rooch_framework::crypto::ed25519::GasParameters::zeros(),
            ecdsa_k1: rooch_framework::crypto::ecdsa_k1::GasParameters::zeros(),
            encoding: rooch_framework::crypto::encoding::GasParameters::zeros(),
            decoding: rooch_framework::crypto::decoding::GasParameters::zeros(),
            bcs: rooch_framework::bcs::GasParameters::zeros(),
        }
    }
}

pub fn all_natives(gas_params: NativeGasParameters) -> NativeFunctionTable {
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
        "encoding",
        rooch_framework::crypto::encoding::make_all(gas_params.encoding)
    );
    add_natives!(
        "decoding",
        rooch_framework::crypto::decoding::make_all(gas_params.decoding)
    );
    add_natives!("bcs", rooch_framework::bcs::make_all(gas_params.bcs));

    let rooch_native_fun_table = make_table_from_iter(ROOCH_FRAMEWORK_ADDRESS, natives);
    native_fun_table.extend(rooch_native_fun_table);

    native_fun_table
}

#[derive(Clone, Debug)]
pub struct RoochGasParameters {
    pub rooch_framework_gas: NativeGasParameters,
    pub instruction_gas: InstructionParameter,
    pub storage_gas: StorageGasParameter,
    pub misc_gas: AbstractValueSizeGasParameter,
}

impl InitialGasSchedule for RoochGasParameters {
    fn initial() -> Self {
        Self {
            rooch_framework_gas: InitialGasSchedule::initial(),
            instruction_gas: initial_instruction_parameter(),
            storage_gas: initial_storage_parameter(),
            misc_gas: initial_misc_parameter(),
        }
    }
}

impl ToOnChainGasSchedule for RoochGasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entires = self.rooch_framework_gas.to_on_chain_gas_schedule();
        entires.extend(instruction_parameter_to_on_chain_gas_schedule(
            self.instruction_gas.clone(),
        ));
        entires.extend(storage_parameter_to_on_chain_gas_schedule(
            self.storage_gas.clone(),
        ));
        entires.extend(misc_parameter_to_on_chain_gas_schedule(
            self.misc_gas.clone(),
        ));
        entires
    }
}

impl FromOnChainGasSchedule for RoochGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            rooch_framework_gas: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)
                .unwrap(),
            instruction_gas: from_on_chain_gas_schedule_to_instruction_parameter(gas_schedule)
                .unwrap(),
            storage_gas: from_on_chain_gas_schedule_to_storage_parameter(gas_schedule).unwrap(),
            misc_gas: from_on_chain_gas_schedule_to_misc_parameter(gas_schedule).unwrap(),
        })
    }
}

impl RoochGasParameters {
    pub fn zeros() -> Self {
        Self {
            rooch_framework_gas: NativeGasParameters::zeros(),
            instruction_gas: InstructionParameter::zeros(),
            storage_gas: StorageGasParameter::zeros(),
            misc_gas: AbstractValueSizeGasParameter::zeros(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GasSchedule {
    pub feature_version: u64,
    pub entries: Vec<(String, u64)>,
}

pub fn default_gas_schedule() -> GasSchedule {
    GasSchedule {
        feature_version: 1,
        entries: RoochGasParameters::initial().to_on_chain_gas_schedule(),
    }
}
