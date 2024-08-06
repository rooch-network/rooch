// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use bitcoin::OutPoint;
use clap::Parser;
use move_vm_types::values::Value;
use rustc_hash::FxHashSet;

use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::MoveState;
use moveos_types::state_resolver::{RootObjectResolver, StatelessResolver};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::error::RoochResult;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::into_address::IntoAddress;
use rooch_types::rooch_network::RoochChainID;

use crate::commands::statedb::commands::{init_job, OutpointInscriptionsMap};
use crate::commands::statedb::commands::inscription::{
    gen_inscription_ids_update, InscriptionSource,
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
        help = "outpoint(original):inscriptions(original inscription_id) map dump path, for debug"
    )]
    pub outpoint_inscriptions_map_dump_path: Option<PathBuf>,
    #[clap(
        long,
        help = "random mode, for randomly select 1/1000 inscriptions & utxos to verify"
    )]
    pub random_mode: bool,

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

        log::info!("indexing and dumping outpoint_inscriptions_map...");
        let (outpoint_inscriptions_map, mapped_outpoint, mapped_inscription, unbound_count) =
            OutpointInscriptionsMap::index_and_dump(
                self.ord_source.clone(),
                self.outpoint_inscriptions_map_dump_path.clone(),
            );
        println!(
            "{} outpoints : {} inscriptions mapped in: {:?} ({} unbound inscriptions ignored)",
            mapped_outpoint,
            mapped_inscription,
            start_time.elapsed(),
            unbound_count
        );

        let outpoint_inscriptions_map = Arc::new(outpoint_inscriptions_map);
        let random_mode = self.random_mode;
        let moveos_store = Arc::new(moveos_store);
        let moveos_store_clone = Arc::clone(&moveos_store);
        // verify inscriptions
        let inscription_source_path = self.ord_source.clone();
        let root_clone_0 = root.clone();
        let verify_inscription_thread = thread::spawn(move || {
            verify_inscription(
                inscription_source_path,
                moveos_store_clone,
                root_clone_0,
                random_mode,
            );
        });
        let moveos_store_clone = Arc::clone(&moveos_store);
        let verify_utxo_thread = thread::spawn(move || {
            verify_utxo(
                self.utxo_source,
                moveos_store_clone,
                root.clone(),
                outpoint_inscriptions_map,
                random_mode,
            );
        });

        verify_inscription_thread.join().unwrap();
        verify_utxo_thread.join().unwrap();

        Ok(())
    }
}

fn verify_utxo(
    input: PathBuf,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    outpoint_inscriptions_map: Arc<OutpointInscriptionsMap>,
    random_mode: bool,
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

    let mut ok_count: u32 = 0;

    for line in reader.by_ref().lines() {
        let line = line.unwrap();
        if is_title_line {
            is_title_line = false;
            if line.starts_with("count") {
                continue;
            }
        }
        let mut utxo_raw = UTXORawData::from_str(&line);
        let (key, state, address_mapping_data) =
            utxo_raw.gen_update(Some(outpoint_inscriptions_map.clone()));
        let addr_updates = if let Some(address_mapping_data) = address_mapping_data {
            address_mapping_data.gen_update(&mut added_address_set)
        } else {
            None
        };
        if random_mode && rand::random::<u32>() % 1000 == 0 {
            let act_utxo_state = resolver
                .get_field_at(utxo_store_state_root, &key)
                .unwrap()
                .unwrap();
            assert_eq!(act_utxo_state, state);
            let act_utxo_value =
                Value::simple_deserialize(&act_utxo_state.value, &UTXO::type_layout()).unwrap();
            let act_utxo = UTXO::from_runtime_value(act_utxo_value).unwrap();
            assert_eq!(utxo_raw.amount, act_utxo.value);
            assert_eq!(utxo_raw.vout, act_utxo.vout);
            assert_eq!(utxo_raw.txid.into_address(), act_utxo.txid);
            let inscriptions =
                outpoint_inscriptions_map.search(&OutPoint::new(utxo_raw.txid, utxo_raw.vout));
            if inscriptions.is_some() {
                let inscriptions = inscriptions.unwrap();
            }
            if addr_updates.is_some() {
                let (addr_key, addr_state) = addr_updates.unwrap();
                let act_address_state = resolver
                    .get_field_at(address_mapping_state_root, &addr_key)
                    .unwrap()
                    .unwrap();
                assert_eq!(act_address_state, addr_state);
            }
        }

        ok_count += 1;
        if ok_count % 1_000_000 == 0 {
            println!("{} utxos verified in {:?}", ok_count, start_time.elapsed());
        }
    }
    assert_eq!(act_utxo_store_state.metadata.size, ok_count as u64);
    assert_eq!(
        act_address_mapping_state.metadata.size,
        added_address_set.len() as u64
    );
    println!(
        "{} utxos verified done in {:?}, utxo_store_meta: {:?}, address_mapping_meta: {:?}",
        ok_count,
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

        let source: InscriptionSource = InscriptionSource::from_str(&line);
        if source.inscription_number < 0 {
            cursed_inscription_count += 1;
        } else {
            blessed_inscription_count += 1;
        }

        if random_mode && rand::random::<u32>() % 1000 == 0 {
            let (key, state, inscription_id) = source.gen_update();
            let act_inscription_state = resolver
                .get_field_at(inscription_store_state_root, &key)
                .unwrap()
                .unwrap();
            assert_eq!(act_inscription_state, state);
            let (key2, state2) = gen_inscription_ids_update(sequence_number, inscription_id);
            let act_inscription_id_state = resolver
                .get_field_at(inscription_store_state_root, &key2)
                .unwrap()
                .unwrap();
            assert_eq!(act_inscription_id_state, state2);
        }

        sequence_number += 1;

        if sequence_number % 1_000_000 == 0 {
            println!(
                "{} inscriptions verified in: {:?}",
                sequence_number,
                start_time.elapsed()
            );
        }
    }

    assert_eq!(
        act_inscription_store_state.metadata.size,
        sequence_number as u64 * 2
    );
    assert_eq!(
        act_inscription_store.cursed_inscription_count,
        cursed_inscription_count
    );
    assert_eq!(
        act_inscription_store.blessed_inscription_count,
        blessed_inscription_count
    );
    assert_eq!(act_inscription_store.next_sequence_number, sequence_number);

    println!(
        "{} inscriptions verified done in {:?}, inscription_store_meta: {:?}",
        sequence_number,
        start_time.elapsed(),
        act_inscription_store_state.metadata
    );
}
