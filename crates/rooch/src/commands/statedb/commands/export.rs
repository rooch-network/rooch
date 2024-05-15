// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::{init_statedb, BATCH_SIZE, STATE_HEADER_PREFIX};
use anyhow::Result;
use clap::Parser;
use csv::Writer;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state_resolver::StatelessResolver;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::AddressMappingWrapper;
use rooch_types::rooch_network::RoochChainID;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

/// Export statedb

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum ExportMode {
    // dump UTXO, Inscription and relative Objects, including AddressMapping object
    #[default]
    Genesis = 0,
    Full = 1,
    Snapshot = 2,
    Object = 3,
}

impl TryFrom<u8> for ExportMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ExportMode::Genesis),
            1 => Ok(ExportMode::Full),
            2 => Ok(ExportMode::Snapshot),
            3 => Ok(ExportMode::Object),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportID {
    pub prefix: String,
    pub object_id: ObjectID,
}

impl ExportID {
    pub fn new(prefix: String, object_id: ObjectID) -> Self {
        Self { prefix, object_id }
    }
}

impl std::fmt::Display for ExportID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.object_id.to_string())
    }
}

impl FromStr for ExportID {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(":");
        let prefix = parts
            .next()
            .ok_or(anyhow::anyhow!("invalid export id"))?
            .to_string();
        let object_id =
            ObjectID::from_str(parts.next().ok_or(anyhow::anyhow!("invalid export id"))?)?;
        Ok(ExportID::new(prefix, object_id))
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
        let mut _context = self.context_options.build()?;
        // let client = context.get_client().await?;

        println!("Start statedb export task, batch_size: {:?}", BATCH_SIZE);
        let (root, moveos_store) = init_statedb(self.base_data_dir.clone(), self.chain_id.clone())?;
        println!("root object: {:?}", root);

        let mut _start_time = SystemTime::now();
        let file_name = self.output.display().to_string();
        let mut writer_builder = csv::WriterBuilder::new();
        let writer_builder = writer_builder.delimiter(b',').double_quote(false);
        let mut writer = writer_builder.from_path(file_name).map_err(|e| {
            RoochError::from(anyhow::Error::msg(format!("Invalid output path: {}", e)))
        })?;
        let root_state_root = self
            .state_root
            .unwrap_or(H256::from(root.state_root.into_bytes()));

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
            ExportMode::Object => {
                let obj_id = self
                    .object_id
                    .expect("Object id should exist in object mode");
                Self::export_object(&moveos_store, root_state_root, obj_id, &mut writer)?;
            }
        }

        println!("Finish export task.");
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
        let address_mapping_id = AddressMappingWrapper::mapping_object_id();
        let reverse_mapping_id = AddressMappingWrapper::reverse_mapping_object_id();
        println!("export_genesis utxo_store_id: {:?}", utxo_store_id);
        println!(
            "export_genesis inscription_store_id: {:?}",
            inscription_store_id
        );
        println!(
            "export_genesis address_mapping_id: {:?}, reverse_mapping_id {:?}",
            address_mapping_id, reverse_mapping_id
        );

        let mut genesis_object_ids = vec![];
        genesis_object_ids.push(utxo_store_id.clone());
        genesis_object_ids.push(inscription_store_id.clone());
        genesis_object_ids.push(address_mapping_id);
        genesis_object_ids.push(reverse_mapping_id);

        let mut genesis_objects = vec![];
        let mut genesis_states = vec![];
        for object_id in genesis_object_ids.into_iter() {
            let state = moveos_store
                .get_field_at(root_state_root, &object_id.to_key())?
                .expect("state should exist.");
            let object = state.clone().as_raw_object()?;
            genesis_states.push((object_id.to_key(), state));
            genesis_objects.push(object);
        }

        // write csv field states
        for obj in genesis_objects.into_iter() {
            Self::export_field_states(
                moveos_store,
                H256::from(obj.state_root.into_bytes()),
                obj.id,
                writer,
            )?;
        }

        // write csv object states.
        {
            let root_export_id = ExportID::new(STATE_HEADER_PREFIX.to_string(), ObjectID::root());
            writer.write_field(root_export_id.to_string())?;
            writer.write_field(format!("{:?}", root_state_root))?;
            writer.write_record(None::<&[u8]>)?;
        }
        for (k, v) in genesis_states.into_iter() {
            writer.write_field(k.to_string())?;
            writer.write_field(v.to_string())?;
            writer.write_record(None::<&[u8]>)?;
        }

        // flush csv writer
        writer.flush()?;
        println!("export_genesis root state_root: {:?}", root_state_root);

        Ok(())
    }

    fn export_object<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        object_id: ObjectID,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        println!("export_object object_id: {:?}", object_id);

        let state = moveos_store
            .get_field_at(root_state_root, &object_id.to_key())?
            .expect("state should exist.");
        let obj = state.clone().as_raw_object()?;

        // write csv field states
        Self::export_field_states(
            moveos_store,
            H256::from(obj.state_root.into_bytes()),
            object_id.clone(),
            writer,
        )?;

        // write csv object states.
        {
            let export_id = ExportID::new(STATE_HEADER_PREFIX.to_string(), object_id.clone());
            writer.write_field(export_id.to_string())?;
            writer.write_field(format!("{:?}", root_state_root))?;
            writer.write_record(None::<&[u8]>)?;
        }
        writer.write_field(object_id.to_key().to_string())?;
        writer.write_field(state.to_string())?;
        writer.write_record(None::<&[u8]>)?;

        // flush csv writer
        writer.flush()?;
        println!("export_object root state_root: {:?}", root_state_root);

        Ok(())
    }

    fn export_field_states<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        state_root: H256,
        object_id: ObjectID,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let starting_key = None;
        let mut count: u64 = 0;

        let mut iter = moveos_store
            .get_state_store()
            .iter(state_root, starting_key.clone())?;

        if object_id.has_child() {
            for item in iter {
                let (_k, v) = item?;
                if v.is_object() {
                    let object = v.clone().as_raw_object()?;
                    if object.size > 0 {
                        Self::export_field_states(
                            moveos_store,
                            H256::from(object.state_root.into_bytes()),
                            object.id,
                            writer,
                        )?;
                    }
                }
            }

            // seek from starting_key
            iter = moveos_store
                .get_state_store()
                .iter(state_root, starting_key.clone())?;
        }

        // write csv header.
        {
            let export_id = ExportID::new(STATE_HEADER_PREFIX.to_string(), object_id.clone());
            writer.write_field(export_id.to_string())?;
            writer.write_field(format!("{:?}", state_root))?;
            writer.write_record(None::<&[u8]>)?;
        }

        for item in iter {
            let (k, v) = item?;
            writer.write_field(k.to_string())?;
            writer.write_field(v.to_string())?;
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
