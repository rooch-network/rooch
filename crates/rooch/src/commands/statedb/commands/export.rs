// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::{init_statedb, BATCH_SIZE};
use anyhow::Error;
use anyhow::Result;
use clap::Parser;
use csv::Writer;
// use move_core_types::effects::Op;
// use moveos_store::state_store::statedb::STATEDB_DUMP_BATCH_SIZE;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
// use moveos_types::moveos_std::object::ObjectID;
// use moveos_types::state::{
//     FieldChange, FieldState, KeyState, MoveState, MoveType, ObjectChange, ObjectState,
// };
use moveos_types::state_resolver::StatelessResolver;
// use proptest_derive::Arbitrary;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::AddressMappingWrapper;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
        // let writer = BufWriter::new(File::open(file_name)?);
        let mut writer_builder = csv::WriterBuilder::new();
        // let writer_builder = writer_builder.delimiter(b'\t').double_quote(false);
        let writer_builder = writer_builder.delimiter(b',').double_quote(false);
        let mut writer = writer_builder
            .from_path(file_name)
            .map_err(|e| RoochError::from(Error::msg(format!("Invalid output path: {}", e))))?;
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
                todo!()
            }
        }

        println!("Finish export task.");
        Ok(())
    }

    fn export_genesis<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        root_state_root: H256,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let utxo_store_id = BitcoinUTXOStore::object_id();
        let inscription_store_id = InscriptionStore::object_id();
        let address_mapping_id = AddressMappingWrapper::mapping_object_id();
        let reverse_mapping_id = AddressMappingWrapper::reverse_mapping_object_id();
        println!("utxo_store_id: {:?}", utxo_store_id);
        println!("inscription_store_id: {:?}", inscription_store_id);
        println!(
            "address_mapping_id: {:?}, reverse_mapping_id {:?}",
            address_mapping_id, reverse_mapping_id
        );

        let utxo_store_obj = moveos_store
            .get_field_at(root_state_root, &utxo_store_id.to_key())?
            .expect("state should exist.")
            .as_raw_object()?;
        let inscription_store_obj = moveos_store
            .get_field_at(root_state_root, &inscription_store_id.to_key())?
            .expect("state should exist.")
            .as_raw_object()?;
        let address_mapping_obj = moveos_store
            .get_field_at(root_state_root, &address_mapping_id.to_key())?
            .expect("state should exist.")
            .as_raw_object()?;
        let reverse_mapping_obj = moveos_store
            .get_field_at(root_state_root, &reverse_mapping_id.to_key())?
            .expect("state should exist.")
            .as_raw_object()?;

        // write csv header.
        {
            writer.write_field("objectstates")?;
            writer.write_field("start")?;
            writer.write_record(None::<&[u8]>)?;
        }

        Self::export_object_fields(
            moveos_store,
            H256::from(utxo_store_obj.state_root.into_bytes()),
            writer,
        )?;
        Self::export_object_fields(
            moveos_store,
            H256::from(inscription_store_obj.state_root.into_bytes()),
            writer,
        )?;
        Self::export_object_fields(
            moveos_store,
            H256::from(address_mapping_obj.state_root.into_bytes()),
            writer,
        )?;
        Self::export_object_fields(
            moveos_store,
            H256::from(reverse_mapping_obj.state_root.into_bytes()),
            writer,
        )?;

        Ok(())
    }

    fn export_object_fields<W: std::io::Write>(
        moveos_store: &MoveOSStore,
        state_root: H256,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let starting_key = None;
        let mut count: u64 = 0;

        let iter = moveos_store
            .get_state_store()
            .iter(state_root, starting_key)?;
        for item in iter {
            let (k, v) = item?;
            // let mut record = vec![];
            writer.write_field(k.to_bytes()?)?;
            writer.write_field(v.to_bytes()?)?;
            // writer.serialize(record)?;
            writer.write_record(None::<&[u8]>)?;

            count += 1;
        }
        // flush csv writer
        writer.flush()?;
        println!("state_root: {:?} export field counts {}", state_root, count);
        Ok(())
    }

    // // Batch dump child object states of specified object by object id
    // fn dump_child_object_states(
    //     &self,
    //     parent_id: ObjectID,
    //     state_root: H256,
    //     starting_key: Option<KeyState>,
    //     with_parent: bool,
    // ) -> Result<(Vec<ObjectState>, Option<KeyState>)> {
    //     let iter = self.iter(state_root, starting_key)?;
    //     let mut data = Vec::new();
    //     let mut counter = 0;
    //     let mut next_key = None;
    //     for item in iter {
    //         if counter >= STATEDB_DUMP_BATCH_SIZE {
    //             break;
    //         };
    //         let (k, v) = item?;
    //         ensure!(k.key_type == ObjectID::type_tag());
    //         let obj_id = ObjectID::from_bytes(k.key.clone())?;
    //         if (with_parent && obj_id == parent_id) || obj_id.is_child(parent_id.clone()) {
    //             let obj = v.as_raw_object()?;
    //             let object_change = ObjectChange::new(Op::New(v));
    //             let object_state = ObjectState::new(
    //                 H256::from(obj.state_root.into_bytes()),
    //                 obj.size,
    //                 obj.id,
    //                 object_change,
    //             );
    //             data.push(object_state);
    //
    //             counter += 1;
    //         };
    //         next_key = Some(k);
    //     }
    //     Ok((data, next_key))
    // }
    //
    // /// Batch dump filed states of specified object by object id
    // fn dump_field_states(
    //     &self,
    //     _object_id: ObjectID,
    //     state_root: H256,
    //     starting_key: Option<KeyState>,
    // ) -> Result<(Vec<FieldState>, Option<KeyState>)> {
    //     let iter = self.iter(state_root, starting_key)?;
    //     let mut data = Vec::new();
    //     let mut next_key = None;
    //     for (counter, item) in iter.enumerate() {
    //         if counter >= STATEDB_DUMP_BATCH_SIZE {
    //             break;
    //         };
    //         let (k, v) = item?;
    //         let field_change = FieldChange::new_normal(Op::New(v));
    //         let field_state = FieldState::new(k.clone(), field_change);
    //         data.push(field_state);
    //
    //         next_key = Some(k);
    //     }
    //     Ok((data, next_key))
    // }
}
