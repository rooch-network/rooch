// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes};
use crate::commands::statedb::commands::init_statedb;
use anyhow::{Error, Result};
use bitcoin::{PublicKey, Txid};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{
    ObjectEntity, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK, SYSTEM_OWNER_ADDRESS,
};
use moveos_types::moveos_std::simple_multimap::SimpleMultiMap;
use moveos_types::moveos_std::table::TablePlaceholder;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{KeyState, MoveState, MoveType, State};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::{BitcoinAddress, MultiChainAddress, RoochAddress};
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::bitcoin::{types, utxo};
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::AddressMappingWrapper;
use rooch_types::into_address::IntoAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use serde::{Deserialize, Serialize};
use smt::UpdateSet;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

pub const BATCH_SIZE: usize = 5000;
pub const SCRIPT_TYPE_P2MS: &str = "p2ms";
pub const SCRIPT_TYPE_P2PK: &str = "p2pk";
pub const SCRIPT_TYPE_NON_STANDARD: &str = "non-standard";

/// Genesis Import UTXO
#[derive(Debug, Parser)]
pub struct GenesisUTXOCommand {
    // #[clap(long, short = 'i', parse(from_os_str))]
    #[clap(long, short = 'i')]
    /// import input file. like ~/.rooch/local/utxo.csv or utxo.csv
    pub input: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXOData {
    /// The txid of the UTXO
    pub txid: String,
    /// The vout of the UTXO
    pub vout: u32,
    pub value: u64,
    pub script: String,
    pub script_type: String,
    pub address: String,
}

impl UTXOData {
    pub fn new(
        txid: String,
        vout: u32,
        value: u64,
        script: String,
        script_type: String,
        address: String,
    ) -> Self {
        Self {
            txid,
            vout,
            value,
            script,
            script_type,
            address,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressMappingData {
    pub origin_address: String,
    pub maddress: MultiChainAddress,
    pub address: AccountAddress,
}

impl AddressMappingData {
    pub fn new(
        origin_address: String,
        maddress: MultiChainAddress,
        address: AccountAddress,
    ) -> Self {
        Self {
            origin_address,
            maddress,
            address,
        }
    }
}

impl GenesisUTXOCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mut _context = self.context_options.build()?;
        // let client = context.get_client().await?;

        println!("Start progress task, batch_size: {:?}", BATCH_SIZE);
        let (root, moveos_store) = init_statedb(self.base_data_dir.clone(), self.chain_id.clone())?;
        let utxo_store_id = BitcoinUTXOStore::object_id();
        let address_mapping_id = AddressMappingWrapper::mapping_object_id();
        let reverse_mapping_object_id = AddressMappingWrapper::reverse_mapping_object_id();
        println!("root object: {:?}", root);
        println!("utxo_store_id: {:?}", utxo_store_id);
        println!(
            "address_mapping_id: {:?}, reverse_mapping_object_id {:?}",
            address_mapping_id, reverse_mapping_object_id
        );

        let mut _start_time = SystemTime::now();
        let file_name = self.input.display().to_string();
        let reader = BufReader::new(File::open(file_name)?);

        let mut address_mapping_checker = HashMap::new();
        let mut utxo_datas = vec![];
        let mut pre_root_state_root = H256::from(root.state_root.into_bytes());
        let mut pre_utxostore_state_root = *GENESIS_STATE_ROOT;
        let mut pre_address_mapping_state_root = *GENESIS_STATE_ROOT;
        let mut pre_reverse_address_mapping_state_root = *GENESIS_STATE_ROOT;

        let mut utxo_count: u64 = 0;
        let mut address_mapping_count: u64 = 0;
        for line in reader.lines() {
            let line = line?;
            // skip the first line
            if line.starts_with("count") {
                continue;
            }
            let str_list: Vec<&str> = line.trim().split(',').collect();
            if str_list.len() != 9 {
                println!("Origin UTXO data format {} error", line);
                std::process::exit(1);
            }
            let txid = str_list[1].to_string();
            let vout = str_list[2]
                .parse::<u32>()
                .map_err(|e| RoochError::from(Error::msg(format!("Invalid vout format: {}", e))))?;
            let amount = str_list[5].parse::<u64>().map_err(|e| {
                RoochError::from(Error::msg(format!("Invalid amount format: {}", e)))
            })?;
            let script = str_list[6].to_string();
            let script_type = str_list[7].to_string();
            let address = str_list[8].to_string();
            let utxo_data = UTXOData::new(txid, vout, amount, script, script_type, address.clone());
            // skip UTXO with type is p2ms or non-standard
            if SCRIPT_TYPE_P2MS.eq(utxo_data.script_type.as_str())
                || SCRIPT_TYPE_NON_STANDARD.eq(utxo_data.script_type.as_str())
            {
                continue;
            }
            if address.is_empty() && !SCRIPT_TYPE_P2PK.eq(utxo_data.script_type.as_str()) {
                println!("Invalid utxo data: {:?}", utxo_data);
                continue;
            }
            utxo_datas.push(utxo_data);

            if utxo_datas.len() >= BATCH_SIZE {
                let (new_utxostore_state_root, address_mapping_datas) = Self::process_utxos(
                    &moveos_store,
                    pre_utxostore_state_root,
                    utxo_datas.clone(),
                )?;
                utxo_datas.clear();
                println!(
                    "process_utxos pre_utxostore_state_root: {:?}, new_utxostore_state_root: {:?}",
                    pre_utxostore_state_root, new_utxostore_state_root
                );
                println!("process_utxos utxo count: {}", utxo_count);
                pre_utxostore_state_root = new_utxostore_state_root;

                let (
                    new_address_mapping_state_root,
                    new_reverse_address_mapping_state_root,
                    incr_count,
                ) = Self::process_address_mappings(
                    &moveos_store,
                    pre_address_mapping_state_root,
                    pre_reverse_address_mapping_state_root,
                    address_mapping_datas,
                    &mut address_mapping_checker,
                )?;
                println!(
                    "process_address_mappings pre_address_mapping_state_root: {:?}, new_address_mapping_state_root: {:?}",
                    pre_address_mapping_state_root, new_address_mapping_state_root
                );
                println!(
                    "process_address_mappings pre_reverse_address_mapping_state_root: {:?}, new_reverse_address_mapping_state_root: {:?}",
                    pre_reverse_address_mapping_state_root, new_reverse_address_mapping_state_root
                );
                pre_address_mapping_state_root = new_address_mapping_state_root;
                pre_reverse_address_mapping_state_root = new_reverse_address_mapping_state_root;
                address_mapping_count += incr_count;
            }
            utxo_count += 1;
        }

        if !utxo_datas.is_empty() {
            let (new_utxostore_state_root, address_mapping_datas) =
                Self::process_utxos(&moveos_store, pre_utxostore_state_root, utxo_datas.clone())?;
            utxo_datas.clear();
            println!(
                "process_utxos pre_utxostore_state_root: {:?}, new_utxostore_state_root: {:?}",
                pre_utxostore_state_root, new_utxostore_state_root
            );
            println!("process_utxos utxo count: {}", utxo_count);
            pre_utxostore_state_root = new_utxostore_state_root;

            let (
                new_address_mapping_state_root,
                new_reverse_address_mapping_state_root,
                incr_count,
            ) = Self::process_address_mappings(
                &moveos_store,
                pre_address_mapping_state_root,
                pre_reverse_address_mapping_state_root,
                address_mapping_datas,
                &mut address_mapping_checker,
            )?;
            println!(
                "process_address_mappings pre_address_mapping_state_root: {:?}, new_address_mapping_state_root: {:?}",
                pre_address_mapping_state_root, new_address_mapping_state_root
            );
            println!(
                "process_address_mappings pre_reverse_address_mapping_state_root: {:?}, new_reverse_address_mapping_state_root: {:?}",
                pre_reverse_address_mapping_state_root, new_reverse_address_mapping_state_root
            );
            // pre_utxostore_state_root = new_utxostore_state_root;
            pre_address_mapping_state_root = new_address_mapping_state_root;
            pre_reverse_address_mapping_state_root = new_reverse_address_mapping_state_root;
            address_mapping_count += incr_count;
        }

        // Update UTXOStore Object
        let mut genesis_utxostore_object = Self::create_genesis_utxostore_object()?;
        genesis_utxostore_object.size += utxo_count;
        genesis_utxostore_object.state_root = pre_utxostore_state_root.into_address();
        let mut update_set = UpdateSet::new();
        let parent_id = BitcoinUTXOStore::object_id();
        update_set.put(parent_id.to_key(), genesis_utxostore_object.into_state());

        // Update Address Mapping Object
        let mut genesis_address_mapping_object = Self::create_genesis_address_mapping_object()?;
        let mut genesis_reverse_address_mapping_object =
            Self::create_genesis_reverse_address_mapping_object()?;
        genesis_address_mapping_object.size += address_mapping_count;
        genesis_address_mapping_object.state_root = pre_address_mapping_state_root.into_address();
        genesis_reverse_address_mapping_object.size += address_mapping_count;
        genesis_reverse_address_mapping_object.state_root =
            pre_reverse_address_mapping_state_root.into_address();

        update_set.put(
            genesis_address_mapping_object.id.to_key(),
            genesis_address_mapping_object.into_state(),
        );
        update_set.put(
            genesis_reverse_address_mapping_object.id.to_key(),
            genesis_reverse_address_mapping_object.into_state(),
        );
        let tree_change_set = apply_fields(&moveos_store, pre_root_state_root, update_set)?;
        apply_nodes(&moveos_store, tree_change_set.nodes)?;
        pre_root_state_root = tree_change_set.state_root;

        // Update Startup Info
        let new_size = root.size;
        let new_startup_info = StartupInfo::new(pre_root_state_root, new_size);
        moveos_store
            .get_config_store()
            .save_startup_info(new_startup_info)?;

        let startup_info = moveos_store.get_config_store().get_startup_info()?;
        println!("New startup_info: {:?}", startup_info);

        println!("Finish progress task.");

        Ok(())
    }

    fn process_utxos(
        moveos_store: &MoveOSStore,
        pre_utxostore_state_root: H256,
        utxo_datas: Vec<UTXOData>,
    ) -> Result<(H256, Vec<AddressMappingData>)> {
        // process utxo statedb
        let utxos_and_address_mapping_datas = utxo_datas
            .into_iter()
            .map(Self::create_utxo_object_and_address_mapping_data)
            .collect::<Result<Vec<_>, _>>()?;

        let mut utxo_update_set = UpdateSet::new();
        let mut adderss_mapping_datas = vec![];
        for (utxo_object, adderss_mapping_data) in utxos_and_address_mapping_datas {
            utxo_update_set.put(utxo_object.id.to_key(), utxo_object.into_state());
            adderss_mapping_datas.push(adderss_mapping_data);
        }
        let utxo_tree_change_set =
            apply_fields(moveos_store, pre_utxostore_state_root, utxo_update_set)?;
        apply_nodes(moveos_store, utxo_tree_change_set.nodes)?;

        Ok((utxo_tree_change_set.state_root, adderss_mapping_datas))
    }

    fn process_address_mappings(
        moveos_store: &MoveOSStore,
        pre_address_mapping_state_root: H256,
        pre_reverse_address_mapping_state_root: H256,
        address_mapping_datas: Vec<AddressMappingData>,
        address_mapping_checker: &mut HashMap<String, bool>,
    ) -> Result<(H256, H256, u64)> {
        // process address mapping statedb
        let mut address_mapping_update_set = UpdateSet::new();
        let mut reverse_address_mapping_update_set = UpdateSet::new();
        let mut nodes = BTreeMap::new();

        let mut incr_count: u64 = 0;
        for adderss_mapping_data in address_mapping_datas {
            if let Entry::Vacant(e) =
                address_mapping_checker.entry(adderss_mapping_data.origin_address)
            {
                address_mapping_update_set.put(
                    adderss_mapping_data.maddress.to_key(),
                    adderss_mapping_data.address.into_state(),
                );

                let reverse_address_mapping_key = KeyState::new(
                    adderss_mapping_data.address.to_bytes(),
                    AccountAddress::type_tag(),
                );

                let reverse_address_mapping_state = State::new(
                    vec![adderss_mapping_data.maddress].to_bytes(),
                    TypeTag::Vector(Box::new(MultiChainAddress::type_tag())),
                );
                reverse_address_mapping_update_set
                    .put(reverse_address_mapping_key, reverse_address_mapping_state);

                e.insert(true);
                incr_count += 1;
            }
        }

        let mut address_mapping_tree_change_set = apply_fields(
            moveos_store,
            pre_address_mapping_state_root,
            address_mapping_update_set,
        )?;
        nodes.append(&mut address_mapping_tree_change_set.nodes);
        let mut reverse_address_mapping_tree_change_set = apply_fields(
            moveos_store,
            pre_reverse_address_mapping_state_root,
            reverse_address_mapping_update_set,
        )?;
        nodes.append(&mut reverse_address_mapping_tree_change_set.nodes);

        apply_nodes(moveos_store, nodes)?;
        Ok((
            address_mapping_tree_change_set.state_root,
            reverse_address_mapping_tree_change_set.state_root,
            incr_count,
        ))
    }

    fn create_utxo_object_and_address_mapping_data(
        mut utxo_data: UTXOData,
    ) -> Result<(ObjectEntity<UTXO>, AddressMappingData)> {
        let txid = Txid::from_str(utxo_data.txid.as_str())?.into_address();

        if SCRIPT_TYPE_P2PK.eq(utxo_data.script_type.as_str()) {
            let pubkey = PublicKey::from_str(utxo_data.script.as_str())?;
            let pubkey_hash = pubkey.pubkey_hash();
            let bitcoin_address = BitcoinAddress::new_p2pkh(&pubkey_hash);
            utxo_data.address = bitcoin_address.to_string();
        }
        let maddress = MultiChainAddress::try_from_str_with_multichain_id(
            RoochMultiChainID::Bitcoin,
            utxo_data.address.as_str(),
        )?;

        let address = AccountAddress::from(RoochAddress::try_from(maddress.clone())?);
        let utxo = UTXO::new(
            txid,
            utxo_data.vout,
            utxo_data.value,
            SimpleMultiMap::create(),
        );

        let out_point = types::OutPoint::new(txid, utxo_data.vout);
        let utxo_id = utxo::derive_utxo_id(&out_point);
        let utxo_object = ObjectEntity::new(utxo_id, address, 0u8, *GENESIS_STATE_ROOT, 0, utxo);
        let address_mapping_data = AddressMappingData::new(utxo_data.address, maddress, address);
        Ok((utxo_object, address_mapping_data))
    }

    fn create_genesis_utxostore_object() -> Result<ObjectEntity<BitcoinUTXOStore>> {
        let utxostore_object = BitcoinUTXOStore { next_tx_index: 0 };
        let utxostore_id = BitcoinUTXOStore::object_id();
        let utxostore_object = ObjectEntity::new(
            utxostore_id,
            SYSTEM_OWNER_ADDRESS,
            SHARED_OBJECT_FLAG_MASK,
            *GENESIS_STATE_ROOT,
            0,
            utxostore_object,
        );
        Ok(utxostore_object)
    }

    fn create_genesis_address_mapping_object() -> Result<ObjectEntity<TablePlaceholder>> {
        let object_id = AddressMappingWrapper::mapping_object_id();
        let address_mapping_object = ObjectEntity::new(
            object_id,
            SYSTEM_OWNER_ADDRESS,
            0u8,
            *GENESIS_STATE_ROOT,
            0,
            TablePlaceholder {
                _placeholder: false,
            },
        );
        Ok(address_mapping_object)
    }

    fn create_genesis_reverse_address_mapping_object() -> Result<ObjectEntity<TablePlaceholder>> {
        let object_id = AddressMappingWrapper::reverse_mapping_object_id();
        let reverse_address_mapping_object = ObjectEntity::new(
            object_id,
            SYSTEM_OWNER_ADDRESS,
            0u8,
            *GENESIS_STATE_ROOT,
            0,
            TablePlaceholder {
                _placeholder: false,
            },
        );
        Ok(reverse_address_mapping_object)
    }
}
