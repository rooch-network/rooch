// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use framework_builder::Stdlib;
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::native_functions::NativeFunction;
use moveos::gas::table::VMGasParameters;
use moveos::moveos::{MoveOS, MoveOSConfig};
use moveos_store::{config_store::ConfigDBStore, MoveOSStore};
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256::H256;
use moveos_types::move_std::ascii::MoveAsciiString;
use moveos_types::moveos_std::gas_schedule::{GasEntry, GasSchedule, GasScheduleConfig};
use moveos_types::moveos_std::object::{ObjectEntity, RootObjectEntity};
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use moveos_types::{h256, state_resolver};
use once_cell::sync::Lazy;
use rooch_framework::natives::gas_parameter::gas_member::{
    FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule,
};
use rooch_framework::ROOCH_FRAMEWORK_ADDRESS;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::error::GenesisError;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use rooch_types::transaction::rooch::RoochTransaction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub static ROOCH_LOCAL_GENESIS: Lazy<RoochGenesis> = Lazy::new(|| {
    let network: RoochNetwork = BuiltinChainID::Local.into();
    RoochGenesis::build(network).expect("build rooch genesis failed")
});

pub struct FrameworksGasParameters {
    pub max_gas_amount: u64,
    pub vm_gas_params: VMGasParameters,
    pub rooch_framework_gas_params: rooch_framework::natives::NativeGasParameters,
    pub bitcoin_move_gas_params: bitcoin_move::natives::GasParameters,
}

impl FrameworksGasParameters {
    pub fn initial() -> Self {
        Self {
            max_gas_amount: GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT,
            vm_gas_params: VMGasParameters::initial(),
            rooch_framework_gas_params: rooch_framework::natives::NativeGasParameters::initial(),
            bitcoin_move_gas_params: bitcoin_move::natives::GasParameters::initial(),
        }
    }

    pub fn to_gas_schedule_config(&self) -> GasScheduleConfig {
        let mut entries = self.vm_gas_params.to_on_chain_gas_schedule();
        entries.extend(self.rooch_framework_gas_params.to_on_chain_gas_schedule());
        entries.extend(self.bitcoin_move_gas_params.to_on_chain_gas_schedule());
        GasScheduleConfig {
            max_gas_amount: self.max_gas_amount,
            entries: entries
                .into_iter()
                .map(|(key, val)| GasEntry {
                    key: MoveAsciiString::from_str(key.as_str())
                        .expect("GasEntry key must be ascii"),
                    val,
                })
                .collect(),
        }
    }

    pub fn load_from_chain(state_resolver: &dyn state_resolver::StateResolver) -> Result<Self> {
        let gas_schedule_state = state_resolver
            .get_object(&GasSchedule::gas_schedule_object_id())?
            .ok_or_else(|| anyhow::anyhow!("Gas schedule object not found"))?;
        let gas_schedule = gas_schedule_state.into_object::<GasSchedule>()?;
        Self::load_from_gas_entries(
            gas_schedule.value.max_gas_amount,
            gas_schedule.value.entries,
        )
    }

    pub fn load_from_gas_config(gas_config: &GasScheduleConfig) -> Result<Self> {
        Self::load_from_gas_entries(gas_config.max_gas_amount, gas_config.entries.clone())
    }

    pub fn load_from_gas_entries(max_gas_amount: u64, entries: Vec<GasEntry>) -> Result<Self> {
        let entries = entries
            .into_iter()
            .map(|entry| (entry.key.to_string(), entry.val))
            .collect::<BTreeMap<_, _>>();
        let vm_gas_parameter = VMGasParameters::from_on_chain_gas_schedule(&entries)
            .ok_or_else(|| anyhow::anyhow!("Failed to load vm gas parameters"))?;
        let rooch_framework_gas_params =
            rooch_framework::natives::NativeGasParameters::from_on_chain_gas_schedule(&entries)
                .ok_or_else(|| anyhow::anyhow!("Failed to load rooch framework gas parameters"))?;
        let bitcoin_move_gas_params =
            bitcoin_move::natives::GasParameters::from_on_chain_gas_schedule(&entries)
                .ok_or_else(|| anyhow::anyhow!("Failed to load bitcoin move gas parameters"))?;
        Ok(Self {
            max_gas_amount,
            vm_gas_params: vm_gas_parameter,
            rooch_framework_gas_params,
            bitcoin_move_gas_params,
        })
    }

    pub fn all_natives(&self) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
        let mut rooch_framework_native_tables =
            rooch_framework::natives::all_natives(self.rooch_framework_gas_params.clone());
        let bitcoin_move_native_table =
            bitcoin_move::natives::all_natives(self.bitcoin_move_gas_params.clone());
        rooch_framework_native_tables.extend(bitcoin_move_native_table);
        rooch_framework_native_tables
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoochGenesis {
    /// The root object after genesis initialization
    pub root: RootObjectEntity,
    pub initial_gas_config: GasScheduleConfig,
    pub genesis_tx: RoochTransaction,
    pub genesis_moveos_tx: MoveOSTransaction,
}

pub enum BuildOption {
    Fresh,
    Release,
}

impl RoochGenesis {
    pub fn build(network: RoochNetwork) -> Result<Self> {
        Self::build_with_option(network, BuildOption::Release)
    }

    pub fn build_with_option(network: RoochNetwork, option: BuildOption) -> Result<Self> {
        let stdlib = match option {
            BuildOption::Fresh => Self::build_stdlib()?,
            BuildOption::Release => Self::load_stdlib()?,
        };
        let genesis_config = network.genesis_config;
        let genesis_ctx = GenesisContext::new(
            network.chain_id.id,
            genesis_config.timestamp,
            genesis_config.sequencer_account,
        );
        let bitcoin_genesis_ctx = BitcoinGenesisContext::new(genesis_config.bitcoin_network);

        let stdlib_package_names = genesis_config.stdlib_package_names.clone();

        let bundles = stdlib.module_bundles(stdlib_package_names.as_slice())?;

        let genesis_tx = RoochTransaction::new_genesis_tx(
            ROOCH_FRAMEWORK_ADDRESS.into(),
            network.chain_id.id,
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

        let gas_parameter = FrameworksGasParameters::initial();
        let gas_config = gas_parameter.to_gas_schedule_config();
        genesis_moveos_tx.ctx.add(genesis_ctx.clone())?;
        genesis_moveos_tx.ctx.add(bitcoin_genesis_ctx.clone())?;
        genesis_moveos_tx.ctx.add(gas_config.clone())?;

        let vm_config = MoveOSConfig::default();

        let moveos = MoveOS::new(
            MoveOSStore::mock_moveos_store()?,
            gas_parameter.all_natives(),
            vm_config,
            vec![],
            vec![],
        )?;
        let (state_root, size, _output) = moveos.init_genesis(genesis_moveos_tx.clone())?;

        Ok(Self {
            root: ObjectEntity::root_object(state_root, size),
            initial_gas_config: gas_config,
            genesis_tx,
            genesis_moveos_tx,
        })
    }

    pub fn genesis_tx(&self) -> RoochTransaction {
        self.genesis_tx.clone()
    }

    pub fn genesis_moveos_tx(&self) -> MoveOSTransaction {
        self.genesis_moveos_tx.clone()
    }

    pub fn genesis_hash(&self) -> H256 {
        h256::sha3_256_of(
            bcs::to_bytes(&self)
                .expect("genesis txs bcs to_bytes should success")
                .as_slice(),
        )
    }

    pub fn genesis_root(&self) -> &RootObjectEntity {
        &self.root
    }

    pub fn genesis_info(&self) -> GenesisInfo {
        GenesisInfo {
            genesis_package_hash: self.genesis_hash(),
            root: self.genesis_root().clone(),
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
                        from_store: Box::new(genesis_info_from_store),
                        from_binary: Box::new(genesis_info_from_binary),
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
        //we load the gas parameter from genesis binary, avoid the code change affect the genesis result
        let genesis_gas_parameter = FrameworksGasParameters::load_from_gas_entries(
            self.initial_gas_config.max_gas_amount,
            self.initial_gas_config.entries.clone(),
        )?;
        let moveos = MoveOS::new(
            moveos_store.clone(),
            genesis_gas_parameter.all_natives(),
            MoveOSConfig::default(),
            vec![],
            vec![],
        )?;

        let (genesis_state_root, size, genesis_tx_output) =
            moveos.init_genesis(self.genesis_moveos_tx())?;

        let inited_root = ObjectEntity::root_object(genesis_state_root, size);
        debug_assert!(
            inited_root == *self.genesis_root(),
            "Genesis state root mismatch"
        );

        //TODO save the genesis txs to sequencer
        let tx_hash = self.genesis_tx().tx_hash();
        moveos_store.handle_tx_output(tx_hash, genesis_state_root, size, genesis_tx_output)?;

        let genesis_info = GenesisInfo::new(self.genesis_hash(), inited_root.clone());
        moveos_store.get_config_store().save_genesis(genesis_info)?;
        Ok(inited_root)
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

static GENESIS_STDLIB_BYTES: &[u8] = include_bytes!("../generated/stdlib");

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
    use moveos_store::MoveOSStore;
    use moveos_types::moveos_std::move_module::ModuleStore;
    use moveos_types::state_resolver::{RootObjectResolver, StateResolver};
    use rooch_types::bitcoin::network::{BitcoinNetwork, Network};
    use rooch_types::rooch_network::BuiltinChainID;

    use crate::FrameworksGasParameters;

    #[test]
    fn test_genesis_init() {
        let _ = tracing_subscriber::fmt::try_init();
        let genesis = super::RoochGenesis::build_with_option(
            BuiltinChainID::Local.into(),
            crate::BuildOption::Fresh,
        )
        .expect("build rooch genesis failed");

        let mut moveos_store = MoveOSStore::mock_moveos_store().unwrap();

        let root = genesis.init_genesis(&mut moveos_store).unwrap();

        let resolver = RootObjectResolver::new(root, &moveos_store);
        let gas_parameter = FrameworksGasParameters::load_from_chain(&resolver)
            .expect("load gas parameter from chain failed");

        assert_eq!(
            FrameworksGasParameters::initial().to_gas_schedule_config(),
            gas_parameter.to_gas_schedule_config()
        );

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
        assert_eq!(bitcoin_network.value.network, Network::Regtest.to_num());
    }
}
