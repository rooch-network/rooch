// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::{config::VMConfig, native_functions::NativeFunction};
use moveos::moveos::MoveOSConfig;
use moveos_stdlib_builder::BuildOptions;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use once_cell::sync::Lazy;

pub static ROOCH_GENESIS: Lazy<RoochGenesis> =
    Lazy::new(|| RoochGenesis::build().expect("build rooch framework failed"));

pub struct RoochGenesis {
    pub config: MoveOSConfig,
    pub stdlib: moveos_stdlib_builder::Stdlib,
    pub gas_params: rooch_framework::natives::GasParameters,
    pub genesis_txs: Vec<MoveOSTransaction>,
}

impl RoochGenesis {
    fn build() -> Result<Self> {
        let stdlib = moveos_stdlib_builder::Stdlib::build(BuildOptions::default())?;
        let bundles =
            moveos_stdlib_builder::Stdlib::build(BuildOptions::default())?.module_bundles()?;
        let genesis_txs = bundles
            .into_iter()
            .map(|(genesis_account, bundle)|
        //TODO make this to RoochTransaction.
        MoveOSTransaction::new_for_test(
            genesis_account,
            MoveAction::ModuleBundle(bundle),
        ))
            .collect();
        let config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };
        let gas_params = rooch_framework::natives::GasParameters::zeros();

        Ok(RoochGenesis {
            config,
            stdlib,
            gas_params,
            genesis_txs,
        })
    }

    pub fn all_natives(&self) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
        rooch_framework::natives::all_natives(self.gas_params.clone())
    }
}
