// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use csv::Writer;
use serde::{Deserialize, Serialize};

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::FieldKey;
use moveos_types::state_resolver::{StateKV, StatelessResolver};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::{
    init_job, GLOBAL_STATE_TYPE_FIELD, GLOBAL_STATE_TYPE_OBJECT, GLOBAL_STATE_TYPE_ROOT,
};

/// Export statedb
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum ExportMode {
    // dump UTXO, Inscription and relative Objects, including RoochToBitcoinAddressMapping object
    #[default]
    Genesis = 0,
    Full = 1,
    Snapshot = 2,
    // rebuild indexer, including UTXO and Inscription
    Indexer = 3,
    Object = 4,
}

impl TryFrom<u8> for ExportMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ExportMode::Genesis),
            1 => Ok(ExportMode::Full),
            2 => Ok(ExportMode::Snapshot),
            3 => Ok(ExportMode::Indexer),
            4 => Ok(ExportMode::Object),
            _ => Err(anyhow::anyhow!(
                "Statedb cli export mode {} is invalid",
                value
            )),
        }
    }
}

impl ExportMode {
    pub fn to_num(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct ExportID {
    pub object_id: ObjectID,
    pub state_root: H256,
    pub parent_state_root: H256,
    pub timestamp: u64,
}

impl ExportID {
    pub fn new(
        object_id: ObjectID,
        state_root: H256,
        parent_state_root: H256,
        timestamp: u64,
    ) -> Self {
        Self {
            object_id,
            state_root,
            parent_state_root,
            timestamp,
        }
    }
}

impl std::fmt::Display for ExportID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_root_str = format!("{:?}", self.state_root);
        let parent_state_root_str = format!("{:?}", self.parent_state_root);
        write!(
            f,
            "{:?}:{}:{}:{}",
            self.object_id, state_root_str, parent_state_root_str, self.timestamp
        )
    }
}

impl FromStr for ExportID {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let object_id =
            ObjectID::from_str(parts.next().ok_or(anyhow::anyhow!("invalid export id"))?)?;
        let state_root = H256::from_str(parts.next().ok_or(anyhow::anyhow!("invalid export id"))?)?;
        let parent_state_root =
            H256::from_str(parts.next().ok_or(anyhow::anyhow!("invalid export id"))?)?;
        let timestamp = parts
            .next()
            .ok_or(anyhow::anyhow!("invalid export id"))?
            .parse::<u64>()?;

        Ok(ExportID::new(
            object_id,
            state_root,
            parent_state_root,
            timestamp,
        ))
    }
}

#[derive(Debug, Parser)]
pub struct ExportCommand {
    /// export state root, default latest state root
    #[clap(long, short = 's')]
    pub state_root: Option<H256>,

    #[clap(long, short = 'o')]
    /// export output file. like ~/.rooch/local/statedb.csv or ./statedb.csv
    pub output: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'm')]
    /// statedb export mode, default is genesis mode
    pub mode: Option<u8>,

    /// export object id, for object mode
    #[clap(long, short = 'i')]
    pub object_id: Option<ObjectID>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl ExportCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (root, moveos_store, start_time) =
            init_job(self.base_data_dir.clone(), self.chain_id.clone());

        let file_name = self.output.display().to_string();
        let mut writer_builder = csv::WriterBuilder::new();
        let writer_builder = writer_builder.delimiter(b',').double_quote(false);
        let mut writer = writer_builder.from_path(file_name).map_err(|e| {
            RoochError::from(anyhow::Error::msg(format!("Invalid output path: {}", e)))
        })?;
        let root_state_root = self.state_root.unwrap_or(root.state_root());

        let mode = ExportMode::try_from(self.mode.unwrap_or(ExportMode::Genesis.to_num()))?;
        match mode {
            ExportMode::Genesis => {
                Self::export_genesis(&moveos_store, root_state_root, &mut writer)?;
            }
            ExportMode::Full => {
                todo!()
            }
            ExportMode::Snapshot => {
                todo!()
            }
            ExportMode::Indexer => {
                Self::export_indexer(&moveos_store, root_state_root, &mut writer)?;
            }
            ExportMode::Object => {
                let obj_id = self
                    .object_id
                    .expect("Object id should exist in object mode");
                Self::export_object(&moveos_store, root_state_root, obj_id, &mut writer)?;
            }
        }

        log::info!("Done in {:?}.", start_time.elapsed(),);
        Ok(())
    }

    /// Field state must be export first, and then object state
    fn export_genesis<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let utxo_store_id = BitcoinUTXOStore::object_id();
        let inscription_store_id = InscriptionStore::object_id();
        let rooch_to_bitcoin_address_mapping_id = RoochToBitcoinAddressMapping::object_id();
        let genesis_object_ids_field_keys = vec![
            utxo_store_id.clone().field_key(),
            inscription_store_id.clone().field_key(),
            rooch_to_bitcoin_address_mapping_id.clone().field_key(),
        ];

        Self::export_genesis_fields(
            moveos_store,
            root_state_root,
            writer,
            genesis_object_ids_field_keys,
        )
    }

    fn export_indexer<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let utxo_store_id = BitcoinUTXOStore::object_id();
        let inscription_store_id = InscriptionStore::object_id();
        let genesis_object_ids_field_keys = vec![
            utxo_store_id.clone().field_key(),
            inscription_store_id.clone().field_key(),
        ];

        Self::export_genesis_fields(
            moveos_store,
            root_state_root,
            writer,
            genesis_object_ids_field_keys,
        )
    }

    fn export_genesis_fields<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        writer: &mut Writer<W>,
        field_keys: Vec<FieldKey>,
    ) -> Result<()> {
        let genesis_states = get_object_states(moveos_store, root_state_root, field_keys);
        // write root state
        {
            let root_export_id =
                ExportID::new(ObjectID::root(), root_state_root, root_state_root, 0);
            writer.write_record([GLOBAL_STATE_TYPE_ROOT, root_export_id.to_string().as_str()])?;
        }
        for (k, v) in genesis_states.into_iter() {
            writer.write_record([k.to_string().as_str(), v.to_string().as_str()])?;
        }

        writer.flush()?;
        Ok(())
    }

    fn export_object<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        object_id: ObjectID,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        println!("export_object object_id: {:?}", object_id);

        let obj = moveos_store
            .get_field_at(root_state_root, &object_id.field_key())?
            .expect("state should exist.");

        let state_root = obj.state_root();
        let timestamp = obj.updated_at();
        // write csv field states
        Self::export_field_states(
            moveos_store,
            state_root,
            root_state_root,
            object_id.clone(),
            false,
            true,
            writer,
        )?;

        // write csv object states.
        {
            let export_id =
                ExportID::new(object_id.clone(), state_root, root_state_root, timestamp);
            writer.write_field(GLOBAL_STATE_TYPE_OBJECT)?;
            writer.write_field(export_id.to_string())?;
            writer.write_record(None::<&[u8]>)?;
        }
        writer.write_field(object_id.field_key().to_string())?;
        //TODO
        //writer.write_field(obj.to_string())?;
        writer.write_record(None::<&[u8]>)?;

        // flush csv writer
        writer.flush()?;
        println!("export_object root state_root: {:?}", root_state_root);

        Ok(())
    }

    fn export_field_states<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        state_root: H256,
        parent_state_root: H256,
        object_id: ObjectID,
        // export child field as object state under indexer mode
        is_child_field_as_object_state: bool,
        is_recursive_export_child_field: bool,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let starting_key = None;
        let mut count: u64 = 0;

        let mut iter = moveos_store
            .get_state_store()
            .iter(state_root, starting_key)?;

        if is_recursive_export_child_field && object_id.has_child() {
            for item in iter {
                let (_k, _v) = item?;
                //TODO
                // if v.is_object() {
                //     let object = v.clone().as_raw_object()?;
                //     if object.size > 0 {
                //         Self::export_field_states(
                //             moveos_store,
                //             object.state_root(),
                //             state_root,
                //             object.id,
                //             false,
                //             false,
                //             writer,
                //         )?;
                //     }
                // }
            }

            // seek from starting_key
            iter = moveos_store
                .get_state_store()
                .iter(state_root, starting_key)?;
        }

        // write csv header.
        {
            let state_type = if is_child_field_as_object_state {
                GLOBAL_STATE_TYPE_OBJECT
            } else {
                GLOBAL_STATE_TYPE_FIELD
            };
            let export_id = ExportID::new(object_id.clone(), state_root, parent_state_root, 0);
            writer.write_field(state_type)?;
            writer.write_field(export_id.to_string())?;
            writer.write_record(None::<&[u8]>)?;
        }

        for item in iter {
            let (k, _v) = item?;
            writer.write_field(k.to_string())?;
            //TODO
            //writer.write_field(v.to_string())?;
            writer.write_record(None::<&[u8]>)?;

            count += 1;
        }

        println!(
            "export_field_states object_id {:?}, state_root: {:?} export field counts {}",
            object_id, state_root, count
        );
        Ok(())
    }
}

fn get_object_states(
    moveos_store: &MoveOSStore,
    state_root: H256,
    object_field_keys: Vec<FieldKey>,
) -> Vec<StateKV> {
    let mut kvs: Vec<StateKV> = Vec::with_capacity(object_field_keys.len());
    for object_field_key in object_field_keys.into_iter() {
        let state = moveos_store
            .get_field_at(state_root, &object_field_key)
            .unwrap()
            .expect("state should exist.");
        kvs.push((object_field_key, state));
    }
    kvs
}
