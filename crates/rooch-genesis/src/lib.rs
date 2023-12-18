// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::{errors::Location, CompiledModule};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::{config::VMConfig, native_functions::NativeFunction};
use moveos::moveos::{MoveOS, MoveOSConfig};
use moveos_stdlib_builder::Stdlib;
use moveos_store::{config_store::ConfigDBStore, MoveOSStore};
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256;
use moveos_types::h256::H256;
use moveos_types::transaction::MoveAction;
use once_cell::sync::Lazy;
use rooch_framework::natives::gas_parameter::gas_member::InitialGasSchedule;
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::GenesisError;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::transaction::rooch::RoochTransaction;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub static ROOCH_LOCAL_GENESIS: Lazy<RoochGenesis> = Lazy::new(|| {
    // genesis for integration test, we need to build the stdlib every time for `private_generic` check
    // see moveos/moveos-verifier/src/metadata.rs#L27-L30
    RoochGenesis::build_with_option(RoochChainID::LOCAL.genesis_ctx(), BuildOption::Fresh)
        .expect("build rooch genesis failed")
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisPackage {
    pub state_root: H256,
    pub genesis_ctx: GenesisContext,
    pub genesis_txs: Vec<RoochTransaction>,
}

#[derive(Clone, Debug)]
pub struct RoochGenesis {
    pub config: MoveOSConfig,
    ///config for the Move integration test
    pub config_for_test: MoveOSConfig,
    //TODO we need to add gas parameters to the GenesisPackage
    //How to serialize the gas parameters?
    pub rooch_framework_gas_params: rooch_framework::natives::GasParameters,
    pub bitcoin_move_gas_params: bitcoin_move::natives::GasParameters,
    pub genesis_package: GenesisPackage,
}

pub enum BuildOption {
    Fresh,
    Release,
}

impl RoochGenesis {
    pub fn build(genesis_ctx: GenesisContext) -> Result<Self> {
        Self::build_with_option(genesis_ctx, BuildOption::Release)
    }

    pub fn build_with_option(genesis_ctx: GenesisContext, option: BuildOption) -> Result<Self> {
        let config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let config_for_test = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let rooch_framework_gas_params = rooch_framework::natives::GasParameters::zeros();
        let bitcoin_move_gas_params = bitcoin_move::natives::GasParameters::zeros();
        let genesis_package = GenesisPackage::build(genesis_ctx, option)?;

        Ok(RoochGenesis {
            config,
            config_for_test,
            rooch_framework_gas_params,
            bitcoin_move_gas_params,
            genesis_package,
        })
    }

    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        self.genesis_package.modules()
    }

    pub fn genesis_txs(&self) -> Vec<RoochTransaction> {
        self.genesis_package.genesis_txs.clone()
    }

    pub fn genesis_ctx(&self) -> GenesisContext {
        self.genesis_package.genesis_ctx.clone()
    }

    pub fn all_natives(&self) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
        let mut rooch_framework_native_tables =
            rooch_framework::natives::all_natives(self.rooch_framework_gas_params.clone());
        let bitcoin_move_native_table =
            bitcoin_move::natives::all_natives(self.bitcoin_move_gas_params.clone());
        rooch_framework_native_tables.extend(bitcoin_move_native_table);
        rooch_framework_native_tables
    }

    pub fn genesis_package_hash(&self) -> H256 {
        h256::sha3_256_of(
            bcs::to_bytes(&self.genesis_package)
                .expect("genesis txs bcs to_bytes should success")
                .as_slice(),
        )
    }

    pub fn genesis_state_root(&self) -> H256 {
        self.genesis_package.state_root
    }

    pub fn genesis_info(&self) -> GenesisInfo {
        GenesisInfo {
            genesis_package_hash: self.genesis_package_hash(),
            state_root_hash: self.genesis_state_root(),
        }
    }

    pub fn check_genesis(&self, config_store: &ConfigDBStore) -> Result<()> {
        let genesis_info_result = config_store.get_genesis();
        match genesis_info_result {
            Ok(Some(genesis_info_from_store)) => {
                let genesis_info_from_binary = self.genesis_info();

                // We need to check the genesis package hash and genesis state root hash
                // because the same genesis package may generate different state root hash when the Move VM is upgraded
                if genesis_info_from_store != genesis_info_from_binary {
                    return Err(GenesisError::GenesisVersionMismatch {
                        from_store: genesis_info_from_store,
                        from_binary: genesis_info_from_binary,
                    }
                    .into());
                }
            }
            Err(e) => return Err(GenesisError::GenesisLoadFailure(e.to_string()).into()),
            Ok(None) => {
                return Err(GenesisError::GenesisNotExist(
                    "genesis hash from store is none".to_string(),
                )
                .into())
            }
        }
        Ok(())
    }
}

static GENESIS_STDLIB_BYTES: &[u8] = include_bytes!("../generated/stdlib");

impl GenesisPackage {
    fn build(genesis_ctx: GenesisContext, build_option: BuildOption) -> Result<Self> {
        let stdlib = match build_option {
            BuildOption::Fresh => Self::build_stdlib()?,
            BuildOption::Release => Self::load_stdlib()?,
        };

        let bundles = stdlib.module_bundles()?;

        let genesis_txs: Vec<RoochTransaction> = bundles
            .into_iter()
            .map(|(genesis_account, bundle)| {
                RoochTransaction::new_genesis_tx(
                    genesis_account.into(),
                    genesis_ctx.chain_id,
                    MoveAction::ModuleBundle(bundle),
                )
            })
            .collect();
        //TODO put gas parameters into genesis package
        let gas_parameters = rooch_framework::natives::GasParameters::initial();
        let vm_config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };
        let mut moveos = MoveOS::new(
            MoveOSStore::mock_moveos_store()?,
            rooch_framework::natives::all_natives(gas_parameters),
            vm_config,
            vec![],
            vec![],
        )?;
        let genesis_result = moveos.init_genesis(genesis_txs.clone(), genesis_ctx.clone())?;
        let state_root = genesis_result
            .last()
            .expect("genesis result should not be empty")
            .0;
        Ok(Self {
            state_root,
            genesis_ctx,
            genesis_txs,
        })
    }

    pub fn build_stdlib() -> Result<Stdlib> {
        rooch_genesis_builder::build_stdlib()
    }

    pub fn load_stdlib() -> Result<Stdlib> {
        moveos_stdlib_builder::Stdlib::decode(GENESIS_STDLIB_BYTES)
    }

    pub fn load_from<P: AsRef<Path>>(genesis_file: P) -> Result<Self> {
        let genesis_package = bcs::from_bytes(&std::fs::read(genesis_file)?)?;
        Ok(genesis_package)
    }

    pub fn save_to<P: AsRef<Path>>(&self, genesis_file: P) -> Result<()> {
        eprintln!("Save genesis to {:?}", genesis_file.as_ref());
        let mut file = File::create(genesis_file)?;
        let contents = bcs::to_bytes(&self)?;
        file.write_all(&contents)?;
        Ok(())
    }

    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        self.genesis_txs
            .iter()
            .filter_map(|tx| {
                if let MoveAction::ModuleBundle(bundle) = &tx.action() {
                    Some(bundle)
                } else {
                    None
                }
            })
            .flatten()
            .map(|module| {
                let compiled_module = CompiledModule::deserialize(module)
                    .map_err(|e| e.finish(Location::Undefined))?;
                Ok(compiled_module)
            })
            .collect::<Result<Vec<CompiledModule>>>()
    }
}

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const MOVE_STD_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../generated/move_std_error_description.errmap");

pub fn move_std_error_descriptions() -> &'static [u8] {
    MOVE_STD_ERROR_DESCRIPTIONS
}

const MOVEOS_STD_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../generated/moveos_std_error_description.errmap");

pub fn moveos_std_error_descriptions() -> &'static [u8] {
    MOVEOS_STD_ERROR_DESCRIPTIONS
}

const ROOCH_FRAMEWORK_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../generated/rooch_framework_error_description.errmap");

pub fn rooch_framework_error_descriptions() -> &'static [u8] {
    ROOCH_FRAMEWORK_ERROR_DESCRIPTIONS
}

#[cfg(test)]
mod tests {
    use moveos::moveos::MoveOS;
    use moveos_store::MoveOSStore;
    use rooch_framework::natives::all_natives;
    use rooch_types::chain_id::RoochChainID;

    #[test]
    fn test_genesis_init() {
        let genesis = super::RoochGenesis::build_with_option(
            RoochChainID::LOCAL.genesis_ctx(),
            crate::BuildOption::Fresh,
        )
        .expect("build rooch framework failed");
        assert_eq!(genesis.genesis_package.genesis_txs.len(), 4);
        let moveos_store = MoveOSStore::mock_moveos_store().unwrap();
        let mut moveos = MoveOS::new(
            moveos_store,
            all_natives(genesis.rooch_framework_gas_params),
            genesis.config,
            vec![],
            vec![],
        )
        .expect("init moveos failed");

        moveos
            .init_genesis(
                genesis.genesis_package.genesis_txs,
                genesis.genesis_package.genesis_ctx,
            )
            .expect("init genesis failed");
    }
}
