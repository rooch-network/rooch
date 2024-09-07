// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::FieldKey;
use moveos_types::state_resolver::StatelessResolver;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::error::RoochResult;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;

use crate::commands::statedb::commands::inscription::{
    gen_inscription_id_update, InscriptionSource,
};
use crate::commands::statedb::commands::utxo::UTXORawData;
use crate::commands::statedb::commands::{init_job, ExportWriter, OutpointInscriptionsMap};

/// Export statedb
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportMode {
    #[default]
    Genesis, // dump InscriptionStore, BitcoinUTXOStore, RoochToBitcoinAddressMapping for genesis start-up
    Full,
    Snapshot,
    Indexer, // dump Full Objects, include InscriptionStore, BitcoinUTXOStore for rebuild indexer
    Object,
}

impl Display for ExportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportMode::Genesis => write!(f, "genesis"),
            ExportMode::Full => write!(f, "full"),
            ExportMode::Snapshot => write!(f, "snapshot"),
            ExportMode::Indexer => write!(f, "indexer"),
            ExportMode::Object => write!(f, "object"),
        }
    }
}

impl FromStr for ExportMode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "genesis" => Ok(ExportMode::Genesis),
            "full" => Ok(ExportMode::Full),
            "snapshot" => Ok(ExportMode::Snapshot),
            "indexer" => Ok(ExportMode::Indexer),
            "object" => Ok(ExportMode::Object),
            _ => Err("export-mode no match"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct ExportID {
    pub object_id: ObjectID,
    pub state_root: H256,
    pub parent_state_root: H256, // If object has no parent, it'll be itself state root.
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

impl Display for ExportID {
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExportObjectName {
    #[default]
    Root,
    UtxoStore,
    InscriptionStore,
    AddressMap,
    Unknown,
}

impl ExportObjectName {
    pub fn object_id(&self) -> Option<ObjectID> {
        match self {
            ExportObjectName::UtxoStore => Some(BitcoinUTXOStore::object_id()),
            ExportObjectName::InscriptionStore => Some(InscriptionStore::object_id()),
            ExportObjectName::AddressMap => Some(RoochToBitcoinAddressMapping::object_id()),
            ExportObjectName::Root => Some(ObjectID::root()),
            ExportObjectName::Unknown => None,
        }
    }
    pub fn from_object_id(object_id: ObjectID) -> Self {
        if object_id == ObjectID::root() {
            ExportObjectName::Root
        } else if object_id == BitcoinUTXOStore::object_id() {
            ExportObjectName::UtxoStore
        } else if object_id == InscriptionStore::object_id() {
            ExportObjectName::InscriptionStore
        } else if object_id == RoochToBitcoinAddressMapping::object_id() {
            ExportObjectName::AddressMap
        } else {
            ExportObjectName::Unknown
        }
    }
}

impl Display for ExportObjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportObjectName::UtxoStore => write!(f, "utxo-store"),
            ExportObjectName::InscriptionStore => write!(f, "inscription-store"),
            ExportObjectName::AddressMap => write!(f, "address-map"),
            ExportObjectName::Root => write!(f, "root"),
            _ => write!(f, "unknown"),
        }
    }
}

impl FromStr for ExportObjectName {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "utxo-store" => Ok(ExportObjectName::UtxoStore),
            "inscription-store" => Ok(ExportObjectName::InscriptionStore),
            "address-map" => Ok(ExportObjectName::AddressMap),
            "root" => Ok(ExportObjectName::Root),
            _ => Ok(ExportObjectName::Unknown),
        }
    }
}

#[allow(dead_code)]
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

    #[clap(long, help = "path to ord source path")]
    pub ord_source_path: Option<PathBuf>,

    #[clap(long, help = "path to utxo source path")]
    pub utxo_source_path: Option<PathBuf>,

    #[clap(long, help = "path to outpoint_inscriptions_map path")]
    pub outpoint_inscriptions_map_path: Option<PathBuf>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'm')]
    /// statedb export mode, default is genesis mode
    pub mode: Option<ExportMode>,

    /// export object id, for object mode
    #[clap(long, short = 'i')]
    pub object_id: Option<ObjectID>,

    #[clap(long)]
    pub object_name: Option<ExportObjectName>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl ExportCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (root, moveos_store, start_time) =
            init_job(self.base_data_dir.clone(), self.chain_id.clone());

        let output = self.output.clone();
        let mut writer = ExportWriter::new(Some(output), None);
        let root_state_root = self.state_root.unwrap_or(root.state_root());

        let mode = self.mode.unwrap_or_default();
        match mode {
            ExportMode::Genesis => {
                todo!()
            }
            ExportMode::Full => {
                todo!()
            }
            ExportMode::Snapshot => {
                todo!()
            }
            ExportMode::Indexer => {
                self.export_indexer(&moveos_store, root_state_root, &mut writer)?;
            }
            ExportMode::Object => {
                let obj_id: ObjectID = self.object_id.unwrap_or_else(|| {
                    self.object_name
                        .expect("object name must be existed if object id not provided")
                        .object_id()
                        .expect("object id must be existed")
                });
                Self::export_object(&moveos_store, root_state_root, obj_id, &mut writer)?;
            }
        }

        writer.flush()?;
        log::info!("Done in {:?}.", start_time.elapsed(),);
        Ok(())
    }

    fn export_indexer(
        &self,
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        writer: &mut ExportWriter,
    ) -> Result<()> {
        // export root_object's top level fields
        let mut object_ids = vec![ObjectID::root()];

        // export utxo_store, inscription_store's top level fields
        if let Some(utxo_path) = self.utxo_source_path.clone() {
            let ord_path = self
                .ord_source_path
                .clone()
                .expect("ord source path must be existed if utxo path is provided");
            let outpoint_inscriptions_map_path = self
                .outpoint_inscriptions_map_path
                .clone()
                .expect("outpoint_inscriptions_map path must be existed if utxo path is provided");
            let utxo_store_object_id = BitcoinUTXOStore::object_id();
            let utxo_store_state_root = get_state_root(
                moveos_store,
                root_state_root,
                utxo_store_object_id.field_key(),
            );
            Self::export_utxo_store(
                utxo_path,
                ord_path.clone(),
                outpoint_inscriptions_map_path,
                utxo_store_state_root,
                writer,
            )?;
            let inscription_store_object_id = InscriptionStore::object_id();
            let inscription_store_state_root = get_state_root(
                moveos_store,
                root_state_root,
                inscription_store_object_id.field_key(),
            );
            Self::export_ord_store(ord_path, inscription_store_state_root, writer)?;
        } else {
            object_ids.push(BitcoinUTXOStore::object_id());
            object_ids.push(InscriptionStore::object_id());
        }

        self.internal_export_indexer(moveos_store, root_state_root, writer, object_ids)?;
        Ok(())
    }

    fn export_ord_store(
        ord_path: PathBuf,
        obj_state_root: H256,
        writer: &mut ExportWriter,
    ) -> Result<()> {
        let start_time = Instant::now();

        let mut reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(ord_path).unwrap());
        let mut is_title_line = true;

        let object_id = InscriptionStore::object_id();
        let object_name = ExportObjectName::from_object_id(object_id.clone()).to_string();

        let mut loop_time = Instant::now();
        let mut sequence_number = 0;
        for line in reader.by_ref().lines() {
            let line = line.unwrap();

            if is_title_line {
                is_title_line = false;
                if line.starts_with("# export at") {
                    // skip block height info
                    continue;
                }
            }

            let source = InscriptionSource::from_str(&line);
            let (key, state, inscription_id) = source.gen_update();
            writer.write_record(&key, &state)?;

            let (k2, v2) = gen_inscription_id_update(sequence_number, inscription_id);
            writer.write_record(&k2, &v2)?;

            sequence_number += 1;
            if sequence_number % 1_000_000 == 0 {
                println!(
                    "exporting top_level_fields of object_id: {:?}({}), exported count: {}. cost: {:?}",
                    object_id, object_name, sequence_number*2, loop_time.elapsed()
                );
                loop_time = Instant::now();
            }
        }

        println!(
            "Done. export_top_level_fields of object_id: {:?}({}), state_root: {:?}, exported count: {}. cost: {:?}",
            object_id,
            object_name,
            obj_state_root,
            sequence_number*2,
            start_time.elapsed()
        );

        Ok(())
    }

    fn export_utxo_store(
        utxo_path: PathBuf,
        ord_path: PathBuf,
        outpoint_inscriptions_map_path: PathBuf,
        obj_state_root: H256,
        writer: &mut ExportWriter,
    ) -> Result<()> {
        let start_time = Instant::now();

        let mut reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(utxo_path).unwrap());
        let mut is_title_line = true;
        let mut max_height = 0;
        let mut count: u64 = 0;

        let outpoint_inscriptions_map =
            OutpointInscriptionsMap::load_or_index(outpoint_inscriptions_map_path, Some(ord_path));
        let outpoint_inscriptions_map = Some(Arc::new(outpoint_inscriptions_map));

        let object_id = BitcoinUTXOStore::object_id();
        let object_name = ExportObjectName::from_object_id(object_id.clone()).to_string();

        let mut loop_time = Instant::now();
        for line in reader.by_ref().lines() {
            let line = line.unwrap();
            if is_title_line {
                is_title_line = false;
                if line.starts_with("count") {
                    continue;
                }
            }

            let mut utxo_raw = UTXORawData::from_str(&line);
            let (key, state) = utxo_raw.gen_utxo_update(outpoint_inscriptions_map.clone());
            writer.write_record(&key, &state)?;
            if utxo_raw.height > max_height {
                max_height = utxo_raw.height;
            }

            count += 1;
            if count % 1_000_000 == 0 {
                println!(
                    "exporting top_level_fields of object_id: {:?}({}), exported count: {}. cost: {:?}",
                    object_id, object_name, count, loop_time.elapsed()
                );
                loop_time = Instant::now();
            }
        }

        println!("utxo max_height: {}", max_height);
        println!(
            "Done. export_top_level_fields of object_id: {:?}({}), state_root: {:?}, exported count: {}. cost: {:?}",
            object_id,
            object_name,
            obj_state_root,
            count,
            start_time.elapsed()
        );
        Ok(())
    }

    fn internal_export_indexer(
        &self,
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        writer: &mut ExportWriter,
        object_ids: Vec<ObjectID>,
    ) -> Result<()> {
        for obj_id in object_ids.into_iter() {
            let state_root = if obj_id == ObjectID::root() {
                root_state_root
            } else {
                get_state_root(moveos_store, root_state_root, obj_id.field_key())
            };
            Self::export_top_level_fields(moveos_store, state_root, obj_id, None, writer)?;
        }
        Ok(())
    }

    pub(crate) fn export_object(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        object_id: ObjectID,
        writer: &mut ExportWriter,
    ) -> Result<()> {
        let state_root = if object_id == ObjectID::root() {
            root_state_root
        } else {
            get_state_root(moveos_store, root_state_root, object_id.field_key())
        };
        Self::export_top_level_fields(moveos_store, state_root, object_id, None, writer)?;
        Ok(())
    }

    // export top level fields of an object, no recursive export child field
    fn export_top_level_fields(
        moveos_store: &MoveOSStore,
        obj_state_root: H256,
        object_id: ObjectID,
        object_name: Option<String>, // human-readable object name for debug
        writer: &mut ExportWriter,
    ) -> Result<()> {
        let start_time = Instant::now();

        let starting_key = None;
        let mut count: u64 = 0;

        let object_name =
            object_name.unwrap_or(ExportObjectName::from_object_id(object_id.clone()).to_string());

        let iter = moveos_store
            .get_state_store()
            .iter(obj_state_root, starting_key)?;

        let mut loop_time = Instant::now();
        for item in iter {
            let (k, v) = item?;
            writer.write_record(&k, &v)?;
            count += 1;
            if count % 1_000_000 == 0 {
                println!(
                    "exporting top_level_fields of object_id: {:?}({}), exported count: {}. cost: {:?}",
                    object_id, object_name, count, loop_time.elapsed()
                );
                loop_time = Instant::now();
            }
        }

        println!(
            "Done. export_top_level_fields of object_id: {:?}({}), state_root: {:?}, exported count: {}. cost: {:?}",
            object_id,
            object_name,
            obj_state_root,
            count,
            start_time.elapsed()
        );
        Ok(())
    }
}

fn get_state_root(
    moveos_store: &MoveOSStore,
    state_root: H256,
    object_field_key: FieldKey,
) -> H256 {
    let state = moveos_store
        .get_field_at(state_root, &object_field_key)
        .unwrap()
        .expect("state must be existed.");
    state.state_root()
}
