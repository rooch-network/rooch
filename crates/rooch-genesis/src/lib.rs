// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::{config::VMConfig, native_functions::NativeFunction};
use moveos::moveos::MoveOSConfig;
use moveos_stdlib_builder::BuildOptions;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use once_cell::sync::Lazy;
use rooch_framework::bindings::transaction_validator;

pub static ROOCH_GENESIS: Lazy<RoochGenesis> =
    Lazy::new(|| RoochGenesis::build().expect("build rooch framework failed"));

pub struct RoochGenesis {
    pub config: MoveOSConfig,
    ///config for the Move integration test
    pub config_for_test: MoveOSConfig,
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
            finalize_function: Some(
                transaction_validator::TransactionValidator::finalize_function_id(),
            ),
        };

        let config_for_test = MoveOSConfig {
            vm_config: VMConfig::default(),
            //We do not execute the finalize function in the test.
            finalize_function: None,
        };

        let gas_params = rooch_framework::natives::GasParameters::zeros();

        Ok(RoochGenesis {
            config,
            config_for_test,
            stdlib,
            gas_params,
            genesis_txs,
        })
    }

    pub fn all_natives(&self) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
        rooch_framework::natives::all_natives(self.gas_params.clone())
    }
}

#[cfg(test)]
mod tests {
    use moveos::moveos::MoveOS;
    use rooch_framework::natives::all_natives;

    #[test]
    fn test_genesis_build() {
        let genesis = super::RoochGenesis::build().expect("build rooch framework failed");
        assert_eq!(genesis.genesis_txs.len(), 3);
    }

    #[test]
    fn test_genesis_init() {
        let genesis = super::RoochGenesis::build().expect("build rooch framework failed");
        let db = moveos_store::MoveOSDB::new_with_memory_store();
        let mut moveos = MoveOS::new(db, all_natives(genesis.gas_params), genesis.config)
            .expect("init moveos failed");
        moveos
            .init_genesis(genesis.genesis_txs)
            .expect("init genesis failed");
    }
}
