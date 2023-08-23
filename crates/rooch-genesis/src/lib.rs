// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::{errors::Location, CompiledModule};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::{config::VMConfig, native_functions::NativeFunction};
use moveos::moveos::MoveOSConfig;
use moveos_stdlib_builder::BuildOptions;
use moveos_store::config_store::ConfigDBStore;
use moveos_types::h256;
use moveos_types::h256::H256;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use once_cell::sync::Lazy;
use rooch_types::error::GenesisError;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub static ROOCH_GENESIS: Lazy<RoochGenesis> =
    Lazy::new(|| RoochGenesis::build().expect("build rooch framework failed"));

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisPackage {
    pub genesis_txs: Vec<MoveOSTransaction>,
}

#[derive(Clone, Debug)]
pub struct RoochGenesis {
    pub config: MoveOSConfig,
    ///config for the Move integration test
    pub config_for_test: MoveOSConfig,
    //TODO we need to add gas parameters to the GenesisPackage
    pub gas_params: rooch_framework::natives::GasParameters,
    pub genesis_package: GenesisPackage,
}

pub enum BuildOption {
    Fresh,
    Release,
}

impl RoochGenesis {
    fn build() -> Result<Self> {
        if cfg!(debug_assertions) {
            Self::build_with_option(BuildOption::Fresh)
        } else {
            Self::build_with_option(BuildOption::Release)
        }
    }

    pub fn build_with_option(option: BuildOption) -> Result<Self> {
        let config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let config_for_test = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let gas_params = rooch_framework::natives::GasParameters::zeros();

        let genesis_package = match option {
            BuildOption::Fresh => GenesisPackage::build()?,
            BuildOption::Release => GenesisPackage::load()?,
        };
        Ok(RoochGenesis {
            config,
            config_for_test,
            gas_params,
            genesis_package,
        })
    }

    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        self.genesis_package.modules()
    }

    pub fn genesis_txs(&self) -> Vec<MoveOSTransaction> {
        self.genesis_package.genesis_txs.clone()
    }

    pub fn all_natives(&self) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
        rooch_framework::natives::all_natives(self.gas_params.clone())
    }

    pub fn genesis_hash(&self) -> H256 {
        h256::sha3_256_of(
            bcs::to_bytes(&self.genesis_txs())
                .expect("genesis txs bcs to_bytes should success")
                .as_slice(),
        )
    }

    pub fn check_genesis(&self, config_store: &ConfigDBStore) -> Result<()> {
        let genesis_hash_result = config_store.get_genesis();
        match genesis_hash_result {
            Ok(Some(genesis_hash_store)) => {
                let genesis_hash = self.genesis_hash();
                if genesis_hash_store != genesis_hash {
                    return Err(GenesisError::GenesisVersionMismatch {
                        expect: genesis_hash_store,
                        real: genesis_hash,
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

static GENESIS_PACKAGE_BYTES: &[u8] = include_bytes!("../genesis/genesis");

impl GenesisPackage {
    pub const GENESIS_FILE_NAME: &'static str = "genesis";
    pub const GENESIS_DIR: &'static str = "genesis";

    fn build() -> Result<Self> {
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
        Ok(Self { genesis_txs })
    }

    pub fn load() -> Result<Self> {
        let genesis_package = bcs::from_bytes(GENESIS_PACKAGE_BYTES)?;
        Ok(genesis_package)
    }

    pub fn load_from<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let genesis_file = data_dir.join(Self::GENESIS_FILE_NAME);
        let genesis_package = bcs::from_bytes(&std::fs::read(genesis_file)?)?;
        Ok(genesis_package)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(path_in_crate(Self::GENESIS_DIR))
    }

    pub fn save_to<P: AsRef<Path>>(&self, data_dir: P) -> Result<()> {
        let data_dir = data_dir.as_ref();
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)?;
        }
        let genesis_file = data_dir.join(Self::GENESIS_FILE_NAME);
        eprintln!("Save genesis to {:?}", genesis_file);
        let mut file = File::create(genesis_file)?;
        let contents = bcs::to_bytes(&self)?;
        file.write_all(&contents)?;
        Ok(())
    }

    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        self.genesis_txs
            .iter()
            .filter_map(|tx| {
                if let MoveAction::ModuleBundle(bundle) = &tx.action {
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

fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}

#[cfg(test)]
mod tests {
    use crate::GenesisPackage;
    use moveos::moveos::MoveOS;
    use moveos_store::MoveOSStore;
    use rooch_framework::natives::all_natives;

    #[test]
    fn test_genesis_package_build_and_save() {
        let genesis_package = GenesisPackage::build().unwrap();
        genesis_package.save().unwrap();
    }

    #[test]
    fn test_genesis_package_load() {
        let _genesis_package = GenesisPackage::load().unwrap();
    }

    #[test]
    fn test_genesis_build() {
        let genesis = super::RoochGenesis::build().expect("build rooch framework failed");
        assert_eq!(genesis.genesis_package.genesis_txs.len(), 3);
    }

    #[test]
    fn test_genesis_hash() {
        let genesis = super::RoochGenesis::build().expect("build rooch framework failed");
        genesis.genesis_hash();
    }

    #[test]
    fn test_genesis_init() {
        let genesis = super::RoochGenesis::build().expect("build rooch framework failed");
        // let db = moveos_store::MoveOSStore::new_with_memory_store();
        let moveos_store = MoveOSStore::mock_moveos_store().unwrap();
        let mut moveos = MoveOS::new(
            moveos_store,
            all_natives(genesis.gas_params),
            genesis.config,
        )
        .expect("init moveos failed");
        moveos
            .init_genesis(genesis.genesis_package.genesis_txs)
            .expect("init genesis failed");
    }
}
