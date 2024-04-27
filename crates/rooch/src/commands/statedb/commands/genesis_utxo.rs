// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes};
use crate::commands::statedb::commands::init_statedb;
use anyhow::{Error, Result};
use bitcoin::Txid;
use clap::Parser;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{
    ObjectEntity, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK, SYSTEM_OWNER_ADDRESS,
};
use moveos_types::moveos_std::simple_multimap::SimpleMultiMap;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::MoveState;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::{MultiChainAddress, RoochAddress};
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::bitcoin::{types, utxo};
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::into_address::IntoAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use serde::{Deserialize, Serialize};
use smt::UpdateSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

pub const BATCH_SIZE: usize = 2000;

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
    pub address: String,
}

impl UTXOData {
    pub fn new(txid: String, vout: u32, value: u64, address: String) -> Self {
        Self {
            txid,
            vout,
            value,
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

        println!("root object: {:?}", root);
        println!("root object id: {:?}", root.id);

        let mut _start_time = SystemTime::now();
        let file_name = self.input.display().to_string();
        let reader = BufReader::new(File::open(file_name)?);

        let mut utxo_datas = vec![];
        let mut pre_root_state_root = H256::from(root.state_root.into_bytes());
        let mut pre_utxostore_state_root = *GENESIS_STATE_ROOT;
        // let mut pre_utxostore_state_root = *GENESIS_STATE_ROOT;
        let mut count: u64 = 0;
        for line in reader.lines() {
            let line = line?;
            println!("{:?}", line);
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
            let vout = str_list[2].parse::<u32>().map_err(|e| {
                RoochError::from(Error::msg(format!(
                    "Invalid vout format: {}",
                    e.to_string()
                )))
            })?;
            let amount = str_list[5].parse::<u64>().map_err(|e| {
                RoochError::from(Error::msg(format!(
                    "Invalid amount format: {}",
                    e.to_string()
                )))
            })?;
            let address = str_list[8].to_string();
            let utxo_data = UTXOData::new(txid, vout, amount, address);
            utxo_datas.push(utxo_data);

            if utxo_datas.len() >= BATCH_SIZE {
                let new_utxostore_state_root = Self::process_utxos(
                    &moveos_store,
                    pre_utxostore_state_root,
                    utxo_datas.clone(),
                )?;
                utxo_datas.clear();
                println!(
                    "process_utxos pre_utxostore_state_root: {:?}, new_utxostore_state_root: {:?}",
                    pre_utxostore_state_root, new_utxostore_state_root
                );
                pre_utxostore_state_root = new_utxostore_state_root;
            }
            count += 1;
        }

        if !utxo_datas.is_empty() {
            let new_utxostore_state_root =
                Self::process_utxos(&moveos_store, pre_utxostore_state_root, utxo_datas.clone())?;
            utxo_datas.clear();
            println!(
                "process_utxos pre_utxostore_state_root: {:?}, new_utxostore_state_root: {:?}",
                pre_utxostore_state_root, new_utxostore_state_root
            );
            pre_utxostore_state_root = new_utxostore_state_root;
        }

        // Update UTXOStore Object
        let mut genesis_utxostore_object = Self::create_genesis_utxostore_object()?;
        genesis_utxostore_object.size += count;
        genesis_utxostore_object.state_root = pre_utxostore_state_root.into_address();
        let mut update_set = UpdateSet::new();
        let parent_id = BitcoinUTXOStore::object_id();
        update_set.put(parent_id.to_key(), genesis_utxostore_object.into_state());
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
    ) -> Result<H256> {
        let utxo_objects = utxo_datas
            .into_iter()
            .map(|v| Self::create_utxo_object(v))
            .collect::<Result<Vec<_>, _>>()?;
        let mut update_set = UpdateSet::new();
        // let parent_id = BitcoinUTXOStore::object_id();
        for utxo_object in utxo_objects {
            update_set.put(utxo_object.id.to_key(), utxo_object.into_state());
        }

        let tree_change_set = apply_fields(moveos_store, pre_utxostore_state_root, update_set)?;
        apply_nodes(moveos_store, tree_change_set.nodes)?;
        Ok(tree_change_set.state_root)
    }

    fn create_utxo_object(utxo_data: UTXOData) -> Result<ObjectEntity<UTXO>> {
        let txid = Txid::from_str(utxo_data.txid.as_str())?.into_address();

        let maddress = MultiChainAddress::try_from_str_with_multichain_id(
            RoochMultiChainID::Bitcoin,
            utxo_data.address.as_str(),
        )?;
        let rooch_address = RoochAddress::try_from(maddress)?;
        let utxo = UTXO::new(
            txid,
            utxo_data.vout,
            utxo_data.value,
            SimpleMultiMap::create(),
        );

        let out_point = types::OutPoint::new(txid, utxo_data.vout);
        let utxo_id = utxo::derive_utxo_id(&out_point);
        let utxo_object = ObjectEntity::new(
            utxo_id,
            rooch_address.into(),
            0u8,
            *GENESIS_STATE_ROOT,
            0,
            utxo,
        );
        Ok(utxo_object)
    }

    fn create_genesis_utxostore_object() -> Result<ObjectEntity<BitcoinUTXOStore>> {
        let utxostore_object = BitcoinUTXOStore { next_tx_index: 0 };
        let utxostore_id = BitcoinUTXOStore::object_id();
        let utxostore_object = ObjectEntity::new(
            utxostore_id,
            SYSTEM_OWNER_ADDRESS.into(),
            SHARED_OBJECT_FLAG_MASK,
            *GENESIS_STATE_ROOT,
            0,
            utxostore_object,
        );
        Ok(utxostore_object)
    }
}
