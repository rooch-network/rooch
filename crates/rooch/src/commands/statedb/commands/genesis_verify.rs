// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use bitcoin::OutPoint;
use clap::Parser;
use move_vm_types::values::Value;
use rustc_hash::FxHashSet;

use moveos_store::MoveOSStore;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::{FieldKey, MoveState, ObjectState};
use moveos_types::state_resolver::{RootObjectResolver, StatelessResolver};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::error::RoochResult;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::into_address::IntoAddress;
use rooch_types::rooch_network::RoochChainID;

use crate::commands::statedb::commands::{
    get_values_by_key, init_job, OutpointInscriptionsMap, UTXO_SEAL_INSCRIPTION_PROTOCOL,
};
use crate::commands::statedb::commands::inscription::{
    derive_inscription_ids, gen_inscription_id_update, InscriptionSource,
};
use crate::commands::statedb::commands::utxo::UTXORawData;

/// Import BTC ordinals & UTXO for genesis
#[derive(Debug, Parser)]
pub struct GenesisVerifyCommand {
    #[clap(long, short = 'i')]
    /// utxo source data file. like ~/.rooch/local/utxo.csv or utxo.csv
    /// The file format is csv, and the first line is the header, the header is as follows:
    /// count, txid, vout, height, coinbase, amount, script, type,address
    pub utxo_source: PathBuf,
    #[clap(long)]
    /// ord source data file. like ~/.rooch/local/ord or ord, ord_input must be sorted by sequence_number
    /// The file format is JSON, and the first line is block height info: # export at block height <N>, ord range: [0, N).
    /// ord_input & utxo_input must be at the same height
    pub ord_source: PathBuf,
    #[clap(
        long,
        help = "outpoint(original):inscriptions(original inscription_id) map dump path"
    )]
    pub outpoint_inscriptions_map_dump_path: PathBuf,
    #[clap(
        long,
        help = "random mode, for randomly select 1/1000 inscriptions & utxos to verify"
    )]
    pub random_mode: bool,
    #[clap(long, help = "mismatched utxo output path")]
    pub utxo_mismatched_output: PathBuf,
    #[clap(long, help = "mismatched ord output path")]
    pub ord_mismatched_output: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GenesisVerifyCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (root, moveos_store, start_time) =
            init_job(self.base_data_dir.clone(), self.chain_id.clone());
        let outpoint_inscriptions_map = self.load_or_index_outpoint_inscription_map(start_time);
        let outpoint_inscriptions_map = Arc::new(outpoint_inscriptions_map);
        let random_mode = self.random_mode;
        let moveos_store = Arc::new(moveos_store);
        let moveos_store_clone = Arc::clone(&moveos_store);
        // verify inscriptions
        let inscription_source_path = self.ord_source.clone();
        let root_clone_0 = root.clone();
        let verify_inscription_thread = thread::Builder::new()
            .name("verify-inscription".to_string())
            .spawn(move || {
                verify_inscription(
                    inscription_source_path,
                    moveos_store_clone,
                    root_clone_0,
                    random_mode,
                    self.ord_mismatched_output,
                );
            })
            .unwrap();
        let moveos_store_clone = Arc::clone(&moveos_store);
        let verify_utxo_thread = thread::Builder::new()
            .name("verify-utxo".to_string())
            .spawn(move || {
                verify_utxo(
                    self.utxo_source,
                    moveos_store_clone,
                    root.clone(),
                    outpoint_inscriptions_map,
                    random_mode,
                    self.utxo_mismatched_output,
                );
            })
            .unwrap();

        verify_inscription_thread.join().unwrap();
        verify_utxo_thread.join().unwrap();

        Ok(())
    }

    fn load_or_index_outpoint_inscription_map(
        &self,
        start_time: Instant,
    ) -> OutpointInscriptionsMap {
        let map_existed = self.outpoint_inscriptions_map_dump_path.exists();
        if map_existed {
            log::info!("load outpoint_inscriptions_map...");
            let outpoint_inscriptions_map =
                OutpointInscriptionsMap::load(self.outpoint_inscriptions_map_dump_path.clone());
            let (outpoint_count, inscription_count) = outpoint_inscriptions_map.stats();
            println!(
                "{} outpoints : {} inscriptions mapped in: {:?}",
                outpoint_count,
                inscription_count,
                start_time.elapsed(),
            );
            outpoint_inscriptions_map
        } else {
            log::info!("indexing and dumping outpoint_inscriptions_map...");
            let (outpoint_inscriptions_map, mapped_outpoint, mapped_inscription, unbound_count) =
                OutpointInscriptionsMap::index_and_dump(
                    self.ord_source.clone(),
                    Some(self.outpoint_inscriptions_map_dump_path.clone()),
                );
            println!(
                "{} outpoints : {} inscriptions mapped in: {:?} ({} unbound inscriptions ignored)",
                mapped_outpoint,
                mapped_inscription,
                start_time.elapsed(),
                unbound_count
            );
            outpoint_inscriptions_map
        }
    }
}

fn verify_utxo(
    input: PathBuf,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    outpoint_inscriptions_map: Arc<OutpointInscriptionsMap>,
    random_mode: bool,
    mismatched_output: PathBuf,
) {
    let start_time = Instant::now();
    let mut reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
    let mut is_title_line = true;
    let mut added_address_set: FxHashSet<String> =
        FxHashSet::with_capacity_and_hasher(60_000_000, Default::default());
    let moveos_store = moveos_store_arc.as_ref();
    let resolver = RootObjectResolver::new(root.clone(), moveos_store);
    let act_utxo_store_state = resolver
        .get_field_at(
            root.state_root(),
            &BitcoinUTXOStore::object_id().field_key(),
        )
        .unwrap()
        .unwrap();

    let act_address_mapping_state = resolver
        .get_field_at(
            root.state_root(),
            &RoochToBitcoinAddressMapping::object_id().field_key(),
        )
        .unwrap()
        .unwrap();

    let utxo_store_state_root = act_utxo_store_state.metadata.state_root.unwrap();
    let address_mapping_state_root = act_address_mapping_state.metadata.state_root.unwrap();

    let file = File::create(mismatched_output.clone()).expect("Unable to create utxo output file");
    let mut output_writer = BufWriter::new(file.try_clone().unwrap());

    let mut total: u32 = 0;
    let mut checked_count: u32 = 0;
    let mut mismatched_count: u32 = 0;
    for line in reader.by_ref().lines() {
        let line = line.unwrap();
        if is_title_line {
            is_title_line = false;
            if line.starts_with("count") {
                continue;
            }
        }
        total += 1;
        if total % 1_000_000 == 0 {
            println!(
                "utxo checking: total: {}, checked: {}, mismatched: {}. cost: {:?}",
                total,
                checked_count,
                mismatched_count,
                start_time.elapsed()
            );
        }
        let mut utxo_raw = UTXORawData::from_str(&line);
        let (key, state) = utxo_raw.gen_utxo_update(Some(outpoint_inscriptions_map.clone()));
        let (_, address_mapping_data) = utxo_raw.gen_address_mapping_data();
        let addr_updates = if let Some(address_mapping_data) = address_mapping_data {
            address_mapping_data.gen_update(&mut added_address_set)
        } else {
            None
        };
        if random_mode && rand::random::<u32>() % 1000 != 0 {
            continue;
        }
        checked_count += 1;
        let act_utxo_state = resolver.get_field_at(utxo_store_state_root, &key).unwrap();
        if act_utxo_state.is_none() {
            writeln!(output_writer, "[utxo] not found: {:?}", utxo_raw)
                .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
        // compare utxo to state_db
        let mut act_utxo_state = act_utxo_state.unwrap();
        clear_metadata(&mut act_utxo_state);
        if act_utxo_state != state {
            writeln!(
                output_writer,
                "[utxo] mismatched state: exp: {:?}, act: {:?}",
                state, act_utxo_state
            )
            .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
        // compare basic value from state_db
        let act_utxo_value =
            Value::simple_deserialize(&act_utxo_state.value, &UTXO::type_layout()).unwrap();
        let act_utxo = UTXO::from_runtime_value(act_utxo_value).unwrap();
        if (utxo_raw.amount != act_utxo.value)
            || (utxo_raw.vout != act_utxo.vout)
            || (utxo_raw.txid.into_address() != act_utxo.txid)
        {
            writeln!(
                output_writer,
                "[utxo] mismatched value: exp: {:?}, act: {:?}",
                utxo_raw, act_utxo
            )
            .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
        // compare seals value from state_db
        let inscriptions =
            outpoint_inscriptions_map.search(&OutPoint::new(utxo_raw.txid, utxo_raw.vout));
        let inscriptions_obj_ids = derive_inscription_ids(inscriptions.clone());
        let act_inscriptions = get_values_by_key(
            act_utxo.seals,
            MoveString::from_str(UTXO_SEAL_INSCRIPTION_PROTOCOL).unwrap(),
        );
        let mismatched_inscription = if inscriptions_obj_ids.is_empty() {
            act_inscriptions.is_some()
        } else {
            act_inscriptions.clone().unwrap() != inscriptions_obj_ids
        };
        if mismatched_inscription {
            writeln!(
                output_writer,
                "[utxo] mismatched inscriptions: exp: {:?}(origin: {:?}), act: {:?}",
                inscriptions_obj_ids, inscriptions, act_inscriptions
            )
            .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
        if addr_updates.is_some() {
            let (addr_key, addr_state) = addr_updates.unwrap();
            let mut act_address_state = resolver
                .get_field_at(address_mapping_state_root, &addr_key)
                .unwrap()
                .unwrap();
            clear_metadata(&mut act_address_state);
            if act_address_state != addr_state {
                writeln!(
                    output_writer,
                    "[address_mapping] mismatched state: exp: {:?}, act: {:?}",
                    addr_state, act_address_state
                )
                .expect("Unable to write line");
                mismatched_count += 1;
                continue;
            }
        }
    }
    output_writer.flush().expect("Unable to flush writer");

    if act_utxo_store_state.metadata.size != total as u64 {
        println!(
            "[utxo_store] mismatched size: exp: {}, act: {}",
            total, act_utxo_store_state.metadata.size
        )
    };
    if act_address_mapping_state.metadata.size != added_address_set.len() as u64 {
        println!(
            "[address_mapping] mismatched size: exp: {}, act: {}",
            added_address_set.len(),
            act_address_mapping_state.metadata.size
        )
    };

    println!(
        "utxo check done. total: {}, checked: {}, mismatched: {}. cost: {:?}, utxo_store_meta: {:?}, address_mapping_meta: {:?}",
        total,
        checked_count,
        mismatched_count,
        start_time.elapsed(),
        act_utxo_store_state.metadata,
        act_address_mapping_state.metadata
    );
}

fn verify_inscription(
    input: PathBuf,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    random_mode: bool,
    mismatched_output: PathBuf,
) {
    let start_time = Instant::now();
    let mut src_reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
    let mut is_title_line = true;
    let mut sequence_number: u32 = 0;
    let mut cursed_inscription_count: u32 = 0;
    let mut blessed_inscription_count: u32 = 0;
    let moveos_store = moveos_store_arc.as_ref();
    let resolver = RootObjectResolver::new(root.clone(), moveos_store);
    let act_inscription_store_state = resolver
        .get_field_at(
            root.state_root(),
            &InscriptionStore::object_id().field_key(),
        )
        .unwrap()
        .unwrap();
    let act_inscription_store_value = Value::simple_deserialize(
        &act_inscription_store_state.value,
        &InscriptionStore::type_layout(),
    )
    .unwrap();
    let act_inscription_store =
        InscriptionStore::from_runtime_value(act_inscription_store_value).unwrap();

    let file = File::create(mismatched_output.clone()).expect("Unable to create utxo output file");
    let mut output_writer = BufWriter::new(file.try_clone().unwrap());

    let mut checked_count: u32 = 0;
    let mut mismatched_count: u32 = 0;

    let inscription_store_state_root = act_inscription_store_state.metadata.state_root.unwrap();
    for line in src_reader.by_ref().lines() {
        let line = line.unwrap();
        if is_title_line {
            is_title_line = false;
            if line.starts_with("# export at") {
                // skip block height info
                continue;
            }
        }
        sequence_number += 1;
        if sequence_number % 1_000_000 == 0 {
            println!(
                "inscription checking: total: {}, checked: {}, mismatched: {}. cost: {:?}",
                sequence_number,
                checked_count,
                mismatched_count,
                start_time.elapsed()
            );
        }

        let source: InscriptionSource = InscriptionSource::from_str(&line);
        if source.inscription_number < 0 {
            cursed_inscription_count += 1;
        } else {
            blessed_inscription_count += 1;
        }

        if random_mode && rand::random::<u32>() % 1000 != 0 {
            continue;
        }
        checked_count += 1;
        let (key, state, inscription_id) = source.gen_update();
        let act_inscription_state = resolver
            .get_field_at(inscription_store_state_root, &key)
            .unwrap();
        if act_inscription_state.is_none() {
            writeln!(output_writer, "[inscription] not found: {:?}", source)
                .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
        let mut act_inscription_state = act_inscription_state.unwrap();
        clear_metadata(&mut act_inscription_state);
        if act_inscription_state != state {
            writeln!(
                output_writer,
                "[inscription] mismatched state: exp: {:?}, act: {:?}",
                state, act_inscription_state
            )
            .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
        let (key2, state2) = gen_inscription_id_update(sequence_number, inscription_id);
        let mut act_inscription_id_state = resolver
            .get_field_at(inscription_store_state_root, &key2)
            .unwrap()
            .unwrap();
        clear_metadata(&mut act_inscription_id_state);
        if act_inscription_id_state != state2 {
            writeln!(
                output_writer,
                "[inscription] mismatched inscription_id state: exp: {:?}, act: {:?}",
                state2, act_inscription_id_state
            )
            .expect("Unable to write line");
            mismatched_count += 1;
            continue;
        }
    }

    output_writer.flush().expect("Unable to flush writer");

    if act_inscription_store_state.metadata.size != sequence_number as u64 * 2
        || act_inscription_store.cursed_inscription_count != cursed_inscription_count
        || act_inscription_store.blessed_inscription_count != blessed_inscription_count
        || act_inscription_store.next_sequence_number != sequence_number
    {
        println!(
            "[inscription_store] mismatched size. metadata: exp: {}, act: {}; cursed: exp: {}, act: {}; blessed: exp: {}, act: {}; next_sequence_number: exp: {}, act: {}",
            sequence_number * 2,
            act_inscription_store_state.metadata.size,
            cursed_inscription_count,
            act_inscription_store.cursed_inscription_count,
            blessed_inscription_count,
            act_inscription_store.blessed_inscription_count,
            sequence_number,
            act_inscription_store.next_sequence_number
        )
    };

    println!(
        "inscription check done. total: {}, checked: {}, mismatched: {}. cost: {:?}, inscription_store_meta: {:?}",
        sequence_number,
        checked_count,
        mismatched_count,
        start_time.elapsed(),
        act_inscription_store_state.metadata,
    );
}

// clear metadata, because it's not deterministic in genesis cmd
fn clear_metadata(state: &mut ObjectState) {
    state.metadata.state_root = None;
    state.metadata.created_at = 0;
    state.metadata.updated_at = 0;
}

#[allow(dead_code)]
fn write_mismatched_state_output(
    output_writer: &mut BufWriter<File>,
    prefix: &str,
    exp: &ObjectState,
    act: &ObjectState,
) {
    writeln!(
        output_writer,
        "{} mismatched state: exp: {}, act: {}",
        prefix, exp, act
    )
    .expect("Unable to write line");
    writeln!(output_writer, "--------------------------------").expect("Unable to write line");
}
