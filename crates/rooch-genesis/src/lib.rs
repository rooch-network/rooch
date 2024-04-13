// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use framework_builder::Stdlib;
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::{config::VMConfig, native_functions::NativeFunction};
use moveos::moveos::{MoveOS, MoveOSConfig};
use moveos_store::{config_store::ConfigDBStore, MoveOSStore};
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{ObjectEntity, RootObjectEntity};
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use once_cell::sync::Lazy;
use rooch_framework::natives::default_gas_schedule;
use rooch_framework::natives::gas_parameter::gas_member::InitialGasSchedule;
use rooch_framework::ROOCH_FRAMEWORK_ADDRESS;
use rooch_types::bitcoin::data_import_config::DataImportMode;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::network::Network;
use rooch_types::error::GenesisError;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::{address::RoochAddress, chain_id::RoochChainID};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub static ROOCH_LOCAL_GENESIS: Lazy<RoochGenesis> = Lazy::new(|| {
    // TODO: For now, ROOCH_LOCAL_GENESIS in only used in integration-test.
    // There is no need to upgrade framework, so we set sequencer to 0x0.
    // Setup sequencer for local genesis if there is only demands.
    let mock_sequencer = RoochAddress::from_str("0x0").expect("parse sequencer address failed");
    // genesis for integration test, we need to build the stdlib every time for `private_generic` check
    // see moveos/moveos-verifier/src/metadata.rs#L27-L30
    let bitcoin_genesis_ctx = BitcoinGenesisContext::new(
        Network::NetworkRegtest.to_num(),
        DataImportMode::None.to_num(),
    );
    let gas_schedule_blob =
        bcs::to_bytes(&default_gas_schedule()).expect("Failure serializing genesis gas schedule");
    RoochGenesis::build_with_option(
        RoochChainID::LOCAL.genesis_ctx(mock_sequencer, gas_schedule_blob),
        bitcoin_genesis_ctx,
        BuildOption::Fresh,
    )
    .expect("build rooch genesis failed")
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisPackage {
    pub state_root: H256,
    pub genesis_ctx: GenesisContext,
    pub bitcoin_genesis_ctx: BitcoinGenesisContext,
    pub genesis_tx: RoochTransaction,
    pub genesis_moveos_tx: MoveOSTransaction,
}

#[derive(Clone, Debug)]
pub struct RoochGenesis {
    pub config: MoveOSConfig,
    ///config for the Move integration test
    pub config_for_test: MoveOSConfig,
    //TODO we need to add gas parameters to the GenesisPackage
    //How to serialize the gas parameters?
    pub rooch_framework_gas_params: rooch_framework::natives::NativeGasParameters,
    pub bitcoin_move_gas_params: bitcoin_move::natives::GasParameters,
    pub genesis_package: GenesisPackage,
}

pub enum BuildOption {
    Fresh,
    Release,
}

impl RoochGenesis {
    pub fn build(
        genesis_ctx: GenesisContext,
        bitcoin_genesis_ctx: BitcoinGenesisContext,
    ) -> Result<Self> {
        Self::build_with_option(genesis_ctx, bitcoin_genesis_ctx, BuildOption::Release)
    }

    pub fn build_with_option(
        genesis_ctx: GenesisContext,
        bitcoin_genesis_ctx: BitcoinGenesisContext,
        option: BuildOption,
    ) -> Result<Self> {
        let config = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let config_for_test = MoveOSConfig {
            vm_config: VMConfig::default(),
        };

        let rooch_framework_gas_params = rooch_framework::natives::NativeGasParameters::initial();
        let bitcoin_move_gas_params = bitcoin_move::natives::GasParameters::initial();
        let genesis_package = GenesisPackage::build(genesis_ctx, bitcoin_genesis_ctx, option)?;

        Ok(RoochGenesis {
            config,
            config_for_test,
            rooch_framework_gas_params,
            bitcoin_move_gas_params,
            genesis_package,
        })
    }

    pub fn genesis_tx(&self) -> RoochTransaction {
        self.genesis_package.genesis_tx.clone()
    }

    pub fn genesis_moveos_tx(&self) -> MoveOSTransaction {
        self.genesis_package.genesis_moveos_tx.clone()
    }

    pub fn genesis_ctx(&self) -> GenesisContext {
        self.genesis_package.genesis_ctx.clone()
    }

    pub fn bitcoin_genesis_ctx(&self) -> BitcoinGenesisContext {
        self.genesis_package.bitcoin_genesis_ctx.clone()
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

    pub fn init_genesis(&self, moveos_store: &mut MoveOSStore) -> Result<RootObjectEntity> {
        let mut moveos = MoveOS::new(
            moveos_store.clone(),
            self.all_natives(),
            self.config.clone(),
            vec![],
            vec![],
        )?;

        let (genesis_state_root, size, genesis_tx_output) =
            moveos.init_genesis(self.genesis_moveos_tx())?;

        debug_assert!(
            genesis_state_root == self.genesis_state_root(),
            "Genesis state root mismatch"
        );

        //TODO should we save the genesis txs to sequencer?
        let tx_hash = self.genesis_tx().tx_hash();
        moveos_store.handle_tx_output(tx_hash, genesis_state_root, size, genesis_tx_output)?;

        let genesis_info = GenesisInfo::new(self.genesis_package_hash(), genesis_state_root);
        moveos_store.get_config_store().save_genesis(genesis_info)?;
        Ok(ObjectEntity::root_object(genesis_state_root, size))
    }
}

static GENESIS_STDLIB_BYTES: &[u8] = include_bytes!("../generated/stdlib");

impl GenesisPackage {
    fn build(
        genesis_ctx: GenesisContext,
        bitcoin_genesis_ctx: BitcoinGenesisContext,
        build_option: BuildOption,
    ) -> Result<Self> {
        let stdlib = match build_option {
            BuildOption::Fresh => Self::build_stdlib()?,
            BuildOption::Release => Self::load_stdlib()?,
        };

        let bundles = stdlib.module_bundles()?;

        let genesis_tx = RoochTransaction::new_genesis_tx(
            ROOCH_FRAMEWORK_ADDRESS.into(),
            genesis_ctx.chain_id,
            //merge all the module bundles into one
            MoveAction::ModuleBundle(
                bundles
                    .into_iter()
                    .flat_map(|(_, bundles)| bundles)
                    .collect(),
            ),
        );

        let mut genesis_moveos_tx = genesis_tx
            .clone()
            .into_moveos_transaction(ObjectEntity::genesis_root_object());

        genesis_moveos_tx.ctx.add(genesis_ctx.clone())?;
        genesis_moveos_tx.ctx.add(bitcoin_genesis_ctx.clone())?;

        //TODO put gas parameters into genesis package
        let gas_parameters = rooch_framework::natives::NativeGasParameters::initial();
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
        let (state_root, _size, _output) = moveos.init_genesis(genesis_moveos_tx.clone())?;

        Ok(Self {
            state_root,
            genesis_ctx,
            bitcoin_genesis_ctx,
            genesis_tx,
            genesis_moveos_tx,
        })
    }

    pub fn build_stdlib() -> Result<Stdlib> {
        rooch_genesis_builder::build_stdlib()
    }

    pub fn load_stdlib() -> Result<Stdlib> {
        framework_builder::Stdlib::decode(GENESIS_STDLIB_BYTES)
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
    use move_core_types::account_address::AccountAddress;
    use moveos::moveos::MoveOS;
    use moveos_store::MoveOSStore;
    use moveos_types::moveos_std::move_module::ModuleStore;
    use moveos_types::moveos_std::object::ObjectEntity;
    use moveos_types::state_resolver::{RootObjectResolver, StateResolver};
    use rooch_framework::natives::{all_natives, default_gas_schedule};
    use rooch_types::bitcoin::data_import_config::DataImportMode;
    use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
    use rooch_types::bitcoin::network::{BitcoinNetwork, Network};
    use rooch_types::chain_id::{BuiltinChainID, RoochChainID};

    #[test]
    fn test_genesis_init() {
        let _ = tracing_subscriber::fmt::try_init();
        let sequencer = AccountAddress::ONE.into();
        let bitcoin_genesis_ctx = BitcoinGenesisContext::new(
            Network::NetworkRegtest.to_num(),
            DataImportMode::None.to_num(),
        );
        let gas_schedule_blob = bcs::to_bytes(&default_gas_schedule())
            .expect("Failure serializing genesis gas schedule");
        let genesis_result = super::RoochGenesis::build_with_option(
            RoochChainID::LOCAL.genesis_ctx(sequencer, gas_schedule_blob),
            bitcoin_genesis_ctx,
            crate::BuildOption::Fresh,
        );
        let genesis = match genesis_result {
            Ok(genesis) => genesis,
            Err(e) => panic!("genesis build failed: {:?}", e),
        };

        let moveos_store = MoveOSStore::mock_moveos_store().unwrap();
        let mut moveos = MoveOS::new(
            moveos_store.clone(),
            all_natives(genesis.rooch_framework_gas_params),
            genesis.config,
            vec![],
            vec![],
        )
        .expect("init moveos failed");

        let (state_root, size, _output) = moveos
            .init_genesis(genesis.genesis_package.genesis_moveos_tx)
            .expect("init genesis failed");
        let root = ObjectEntity::root_object(state_root, size);
        let resolver = RootObjectResolver::new(root, &moveos_store);

        let module_store_state = resolver
            .get_object(&ModuleStore::module_store_id())
            .unwrap();
        assert!(module_store_state.is_some());
        let _module_store_obj = module_store_state
            .unwrap()
            .into_object::<ModuleStore>()
            .unwrap();
        let chain_id_state = resolver
            .get_object(&rooch_types::framework::chain_id::ChainID::chain_id_object_id())
            .unwrap();
        assert!(chain_id_state.is_some());
        let chain_id = chain_id_state
            .unwrap()
            .into_object::<rooch_types::framework::chain_id::ChainID>()
            .unwrap();
        assert_eq!(chain_id.value.id, BuiltinChainID::Local.chain_id().id());
        let bitcoin_network_state = resolver
            .get_object(&rooch_types::bitcoin::network::BitcoinNetwork::object_id())
            .unwrap();
        assert!(bitcoin_network_state.is_some());
        let bitcoin_network = bitcoin_network_state
            .unwrap()
            .into_object::<BitcoinNetwork>()
            .unwrap();
        assert_eq!(
            bitcoin_network.value.network,
            Network::NetworkRegtest.to_num()
        );
    }
}
