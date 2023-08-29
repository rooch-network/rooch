// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::{errors::Location, CompiledModule};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_package::BuildConfig;
use move_vm_runtime::{config::VMConfig, native_functions::NativeFunction};
use moveos::moveos::{MoveOS, MoveOSConfig};
use moveos_stdlib_builder::{Stdlib, StdlibBuildConfig};
use moveos_store::{config_store::ConfigDBStore, MoveOSStore};
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256;
use moveos_types::h256::H256;
use moveos_types::transaction::MoveAction;
use once_cell::sync::Lazy;
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::GenesisError;
use rooch_types::framework::genesis;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::transaction::rooch::RoochTransaction;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub static ROOCH_DEV_GENESIS: Lazy<RoochGenesis> = Lazy::new(|| {
    RoochGenesis::build(RoochChainID::DEV.chain_id().id()).expect("build rooch genesis failed")
});

static ROOCH_FRESH_STDLIB: Lazy<Stdlib> =
    Lazy::new(|| GenesisPackage::build_stdlib().expect("build rooch stdlib failed"));

static ROOCH_RELEASE_STDLIB: Lazy<Stdlib> =
    Lazy::new(|| GenesisPackage::load_stdlib().expect("load rooch stdlib failed"));

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
    pub gas_params: rooch_framework::natives::GasParameters,
    pub genesis_package: GenesisPackage,
}

pub enum BuildOption {
    Fresh,
    Release,
}

impl RoochGenesis {
    pub fn build(chain_id: u64) -> Result<Self> {
        if cfg!(debug_assertions) {
            Self::build_with_option(chain_id, BuildOption::Fresh)
        } else {
            Self::build_with_option(chain_id, BuildOption::Release)
        }
    }

    pub fn build_with_option(chain_id: u64, option: BuildOption) -> Result<Self> {
        let config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let config_for_test = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let gas_params = rooch_framework::natives::GasParameters::zeros();
        let genesis_package = GenesisPackage::build(chain_id, option)?;

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

    pub fn genesis_txs(&self) -> Vec<RoochTransaction> {
        self.genesis_package.genesis_txs.clone()
    }

    pub fn genesis_ctx(&self) -> GenesisContext {
        self.genesis_package.genesis_ctx.clone()
    }

    pub fn all_natives(&self) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
        rooch_framework::natives::all_natives(self.gas_params.clone())
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

static GENESIS_STDLIB_BYTES: &[u8] = include_bytes!("../genesis/stdlib");

static STDLIB_BUILD_CONFIGS: Lazy<Vec<StdlibBuildConfig>> = Lazy::new(|| {
    let move_stdlib_path = path_in_crate("../../moveos/moveos-stdlib/move-stdlib")
        .canonicalize()
        .expect("canonicalize path failed");
    let moveos_stdlib_path = path_in_crate("../../moveos/moveos-stdlib/moveos-stdlib")
        .canonicalize()
        .expect("canonicalize path failed");
    let rooch_framework_path = path_in_crate("../rooch-framework")
        .canonicalize()
        .expect("canonicalize path failed");
    vec![
        StdlibBuildConfig {
            path: move_stdlib_path.clone(),
            error_prefix: "E".to_string(),
            error_code_map_output_file: move_stdlib_path.join("error_description.errmap"),
            document_template: move_stdlib_path.join("doc_template/README.md"),
            document_output_directory: move_stdlib_path.join("doc"),
            build_config: BuildConfig::default(),
        },
        StdlibBuildConfig {
            path: moveos_stdlib_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: moveos_stdlib_path.join("error_description.errmap"),
            document_template: moveos_stdlib_path.join("doc_template/README.md"),
            document_output_directory: moveos_stdlib_path.join("doc"),
            build_config: BuildConfig::default(),
        },
        StdlibBuildConfig {
            path: rooch_framework_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: rooch_framework_path.join("error_description.errmap"),
            document_template: rooch_framework_path.join("doc_template/README.md"),
            document_output_directory: rooch_framework_path.join("doc"),
            build_config: BuildConfig::default(),
        },
    ]
});

impl GenesisPackage {
    pub const GENESIS_FILE_NAME: &'static str = "genesis";
    pub const STDLIB_FILE_NAME: &'static str = "genesis/stdlib";
    pub const GENESIS_DIR: &'static str = "genesis";

    fn build(chain_id: u64, build_option: BuildOption) -> Result<Self> {
        let stdlib = match build_option {
            BuildOption::Fresh => ROOCH_FRESH_STDLIB.clone(),
            BuildOption::Release => ROOCH_RELEASE_STDLIB.clone(),
        };

        let bundles = stdlib.module_bundles()?;

        let genesis_txs: Vec<RoochTransaction> = bundles
            .into_iter()
            .map(|(genesis_account, bundle)| {
                RoochTransaction::new_genesis_tx(
                    genesis_account.into(),
                    chain_id,
                    MoveAction::ModuleBundle(bundle),
                )
            })
            .collect();
        //TODO put gas parameters into genesis package
        let gas_parameters = rooch_framework::natives::GasParameters::zeros();
        let vm_config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };
        let mut moveos = MoveOS::new(
            MoveOSStore::mock_moveos_store()?,
            rooch_framework::natives::all_natives(gas_parameters),
            vm_config,
        )?;
        let genesis_ctx = genesis::GenesisContext::new(chain_id);
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
        moveos_stdlib_builder::Stdlib::build(STDLIB_BUILD_CONFIGS.clone())
    }

    pub fn load_stdlib() -> Result<Stdlib> {
        moveos_stdlib_builder::Stdlib::decode(GENESIS_STDLIB_BYTES)
    }

    pub fn stdlib_file() -> PathBuf {
        path_in_crate(Self::STDLIB_FILE_NAME)
    }

    pub fn load_from<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let genesis_file = data_dir.join(Self::GENESIS_FILE_NAME);
        let genesis_package = bcs::from_bytes(&std::fs::read(genesis_file)?)?;
        Ok(genesis_package)
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

pub(crate) fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[cfg(test)]
mod tests {
    use moveos::moveos::MoveOS;
    use moveos_store::MoveOSStore;
    use rooch_framework::natives::all_natives;
    use rooch_types::chain_id::RoochChainID;

    #[test]
    fn test_genesis_build() {
        let genesis = super::RoochGenesis::build(RoochChainID::DEV.chain_id().id())
            .expect("build rooch framework failed");
        genesis.genesis_package_hash();
        assert_eq!(genesis.genesis_package.genesis_txs.len(), 3);
    }

    #[test]
    fn test_genesis_init() {
        let genesis = super::RoochGenesis::build(RoochChainID::DEV.chain_id().id())
            .expect("build rooch framework failed");
        let moveos_store = MoveOSStore::mock_moveos_store().unwrap();
        let mut moveos = MoveOS::new(
            moveos_store,
            all_natives(genesis.gas_params),
            genesis.config,
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
