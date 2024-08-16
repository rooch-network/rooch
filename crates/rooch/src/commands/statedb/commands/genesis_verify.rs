// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use bitcoin::OutPoint;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use move_vm_types::values::Value;
use rustc_hash::FxHashSet;

use bitcoin_move::natives::ord::inscription_id::InscriptionId;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::{DynamicField, ObjectMeta};
use moveos_types::state::{MoveState, MoveStructState, ObjectState};
use moveos_types::state_resolver::{RootObjectResolver, StatelessResolver};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::ord::{Inscription, InscriptionID, InscriptionStore};
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::error::RoochResult;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;

use crate::commands::statedb::commands::inscription::{
    gen_inscription_id_update, InscriptionSource,
};
use crate::commands::statedb::commands::utxo::{AddressMappingData, UTXORawData};
use crate::commands::statedb::commands::{init_job, OutpointInscriptionsMap};

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
        help = "random mode, for randomly select 1/sample_rate inscriptions & utxos to verify"
    )]
    pub random_mode: bool,
    #[clap(
        long,
        help = "sample rate, for randomly select 1/sample_rate inscriptions & utxos to verify",
        default_value = "1000"
    )]
    pub sample_rate: u32,
    #[clap(long, help = "mismatched utxo output path")]
    pub utxo_mismatched_output: PathBuf,
    #[clap(long, help = "mismatched ord output path")]
    pub ord_mismatched_output: PathBuf,
    #[clap(
        long,
        help = "inscription cases must be verified, file is outpoint list"
    )]
    pub utxo_cases: Option<PathBuf>,
    #[clap(
        long,
        help = "utxo cases must be verified, file is inscription_number list"
    )]
    pub ord_cases: Option<PathBuf>,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

struct UTXOCases {
    cases: HashSet<OutPoint>,
}

impl UTXOCases {
    fn load(path: Option<PathBuf>) -> Self {
        let path = match path {
            None => {
                return Self {
                    cases: HashSet::new(),
                }
            }
            Some(path) => path,
        };
        let mut cases = HashSet::new();
        let file = File::open(path).expect("Unable to open utxo cases file");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let outpoint = OutPoint::from_str(&line).unwrap();
            cases.insert(outpoint);
        }
        Self { cases }
    }
    fn contains(&self, outpoint: &OutPoint) -> bool {
        self.cases.contains(outpoint)
    }
}

struct OrdCases {
    cases: HashSet<u32>,
}

impl OrdCases {
    fn load(path: Option<PathBuf>) -> Self {
        let path = match path {
            None => {
                return Self {
                    cases: HashSet::new(),
                }
            }
            Some(path) => path,
        };
        let mut cases = HashSet::new();
        let file = File::open(path).expect("Unable to open ord cases file");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let ord = line.parse::<u32>().unwrap();
            cases.insert(ord);
        }
        Self { cases }
    }
    fn contains(&self, inscription_number: u32) -> bool {
        self.cases.contains(&inscription_number)
    }
}

impl GenesisVerifyCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (root, moveos_store, _) = init_job(self.base_data_dir.clone(), self.chain_id.clone());
        let outpoint_inscriptions_map = OutpointInscriptionsMap::load_or_index(
            self.outpoint_inscriptions_map_dump_path,
            self.ord_source.clone(),
        );
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
                    self.ord_cases,
                    moveos_store_clone,
                    root_clone_0,
                    random_mode,
                    self.sample_rate,
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
                    self.utxo_cases,
                    moveos_store_clone,
                    root.clone(),
                    outpoint_inscriptions_map,
                    random_mode,
                    self.sample_rate,
                    self.utxo_mismatched_output,
                );
            })
            .unwrap();

        verify_inscription_thread.join().unwrap();
        verify_utxo_thread.join().unwrap();

        Ok(())
    }
}

fn verify_utxo(
    input: PathBuf,
    case_path: Option<PathBuf>,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    outpoint_inscriptions_map: Arc<OutpointInscriptionsMap>,
    random_mode: bool,
    sample_rate: u32,
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
    let mut output_writer = BufWriter::with_capacity(1 << 23, file.try_clone().unwrap());

    let cases = UTXOCases::load(case_path);

    let mut utxo_total: u32 = 0;
    let mut utxo_checked_count: u32 = 0;
    let mut utxo_mismatched_count: u32 = 0;
    let mut utxo_not_found_count: u32 = 0;

    let mut address_checked_count: u32 = 0;
    let mut address_mismatched_count: u32 = 0;
    let mut address_not_found_count: u32 = 0;

    for line in reader.by_ref().lines() {
        let line = line.unwrap();
        if is_title_line {
            is_title_line = false;
            if line.starts_with("count") {
                continue;
            }
        }

        // have to parse every line for get address_mapping count
        let mut utxo_raw = UTXORawData::from_str(&line);
        let (_, exp_addr_map) = utxo_raw.gen_address_mapping_data();
        let addr_updates = if let Some(address_mapping_data) = exp_addr_map {
            address_mapping_data.gen_update(&mut added_address_set)
        } else {
            None
        };

        utxo_total += 1;
        if utxo_total % 1_000_000 == 0 {
            println!(
                "utxo checking: total: {}. (mismatched(not_found)/checked): utxo: ({}({})/{}); address: ({}({})/{}). cost: {:?}",
                utxo_total,
                utxo_mismatched_count,
                utxo_not_found_count,
                utxo_checked_count,
                address_mismatched_count,
                address_not_found_count,
                address_checked_count,
                start_time.elapsed()
            );
        }

        let is_case = cases.contains(&OutPoint {
            txid: utxo_raw.txid,
            vout: utxo_raw.vout,
        });

        if (random_mode && rand::random::<u32>() % sample_rate != 0) && !is_case {
            continue;
        }
        // check utxo
        utxo_checked_count += 1;
        let (exp_utxo_key, exp_utxo_state) =
            utxo_raw.gen_utxo_update(Some(outpoint_inscriptions_map.clone()));
        let act_utxo_state = resolver
            .get_field_at(utxo_store_state_root, &exp_utxo_key)
            .unwrap();
        let (mismatched, not_found) = write_mismatched_state_output::<UTXO, UTXORawData>(
            &mut output_writer,
            "[utxo]",
            exp_utxo_state,
            act_utxo_state.clone(),
            Some(utxo_raw.clone()),
        );
        if mismatched {
            utxo_mismatched_count += 1;
        }
        if not_found {
            utxo_not_found_count += 1;
        }
        // check address
        if addr_updates.is_some() {
            address_checked_count += 1;
            let (exp_addr_key, exp_addr_state) = addr_updates.unwrap();
            let act_address_state = resolver
                .get_field_at(address_mapping_state_root, &exp_addr_key)
                .unwrap();
            let (mismatched, not_found) = write_mismatched_state_output::<
                DynamicField<AccountAddress, BitcoinAddress>,
                AddressMappingData,
            >(
                &mut output_writer,
                "[address_mapping]",
                exp_addr_state,
                act_address_state.clone(),
                None,
            );
            if mismatched {
                address_mismatched_count += 1;
            }
            if not_found {
                address_not_found_count += 1;
            }
        }
    }
    output_writer.flush().expect("Unable to flush writer");

    let mut result = "OK";
    if act_utxo_store_state.metadata.size != utxo_total as u64 {
        result = "FAILED";
        println!("------------FAILED----------------");
        println!(
            "[utxo_store] mismatched metadata.size: exp: {}, act: {}",
            utxo_total, act_utxo_store_state.metadata.size
        )
    };
    if act_address_mapping_state.metadata.size != added_address_set.len() as u64 {
        result = "FAILED";
        println!("------------FAILED----------------");
        println!(
            "[address_mapping] mismatched metadata.size: exp: {}, act: {}",
            added_address_set.len(),
            act_address_mapping_state.metadata.size
        )
    };
    if utxo_mismatched_count != 0 || address_mismatched_count != 0 {
        result = "FAILED";
    }
    println!("------------{}----------------", result);
    println!(
        "utxo check {}. total: {}. (mismatched(not_found)/checked): utxo: ({}({})/{}); address: ({}({})/{}). cost: {:?}",
        result,
        utxo_total,
        utxo_mismatched_count,
        utxo_not_found_count,
        utxo_checked_count,
        address_mismatched_count,
        address_not_found_count,
        address_checked_count,
        start_time.elapsed()
    );
}

fn verify_inscription(
    input: PathBuf,
    case_path: Option<PathBuf>,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    random_mode: bool,
    sample_rate: u32,
    mismatched_output: PathBuf,
) {
    let start_time = Instant::now();
    let mut src_reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
    let mut is_title_line = true;
    let mut total: u32 = 0;
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

    let file =
        File::create(mismatched_output.clone()).expect("Unable to create inscription output file");
    let mut output_writer = BufWriter::with_capacity(1 << 23, file.try_clone().unwrap());

    let cases = OrdCases::load(case_path);

    let mut checked_count: u32 = 0;
    let mut mismatched_count: u32 = 0;
    let mut not_found_count: u32 = 0;
    let mut mismatched_inscription_id_count: u32 = 0;
    let mut not_found_inscription_id_count: u32 = 0;

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

        // have to parse every line for get cursed/blessed count
        let source: InscriptionSource = InscriptionSource::from_str(&line);
        if source.inscription_number < 0 {
            cursed_inscription_count += 1;
        } else {
            blessed_inscription_count += 1;
        }

        total += 1;
        if total % 1_000_000 == 0 {
            println!(
                "inscription checking: total: {}. (mismatched(not_found)/checked): inscription: ({}({})/{}); inscription_id: ({}({})/{}). cost: {:?}",
                total,
                mismatched_count,
                not_found_count,
                checked_count,
                mismatched_inscription_id_count,
                not_found_inscription_id_count,
                checked_count,
                start_time.elapsed()
            );
        }

        let is_case = cases.contains(source.inscription_number as u32);

        if (random_mode && rand::random::<u32>() % sample_rate != 0) && !is_case {
            continue;
        }
        // check inscription
        checked_count += 1;
        let (exp_key, exp_state, exp_inscription_id) = source.gen_update();
        let act_inscription_state = resolver
            .get_field_at(inscription_store_state_root, &exp_key)
            .unwrap();
        let (mismatched, not_found) = write_mismatched_state_output::<Inscription, InscriptionSource>(
            &mut output_writer,
            "[inscription]",
            exp_state,
            act_inscription_state.clone(),
            Some(source.clone()),
        );
        if mismatched {
            mismatched_count += 1;
        }
        if not_found {
            not_found_count += 1;
        }
        // check inscription_id
        let (exp_inscription_id_key, exp_inscription_id_state) =
            gen_inscription_id_update(total - 1, exp_inscription_id.clone());
        let act_inscription_id_state = resolver
            .get_field_at(inscription_store_state_root, &exp_inscription_id_key)
            .unwrap();
        let (mismatched, not_found) =
            write_mismatched_state_output::<DynamicField<u32, InscriptionID>, InscriptionId>(
                &mut output_writer,
                "[inscription_id]",
                exp_inscription_id_state,
                act_inscription_id_state.clone(),
                Some(source.id),
            );
        if mismatched {
            mismatched_inscription_id_count += 1;
        }
        if not_found {
            not_found_inscription_id_count += 1;
        }
    }

    output_writer.flush().expect("Unable to flush writer");

    let mut result = "OK";
    if act_inscription_store_state.metadata.size != total as u64 * 2
        || act_inscription_store.cursed_inscription_count != cursed_inscription_count
        || act_inscription_store.blessed_inscription_count != blessed_inscription_count
        || act_inscription_store.next_sequence_number != total
    {
        result = "FAILED";
        println!("------------FAILED----------------");
        println!(
            "[inscription_store] mismatched. metadata.size: exp: {}, act: {}; cursed: exp: {}, act: {}; blessed: exp: {}, act: {}; next_sequence_number: exp: {}, act: {}",
            total * 2,
            act_inscription_store_state.metadata.size,
            cursed_inscription_count,
            act_inscription_store.cursed_inscription_count,
            blessed_inscription_count,
            act_inscription_store.blessed_inscription_count,
            total,
            act_inscription_store.next_sequence_number
        )
    };
    if mismatched_count != 0 || mismatched_inscription_id_count != 0 {
        result = "FAILED";
    }

    println!("-----------{}-----------------", result);
    println!(
        "inscription check {}. total: {}. (mismatched(not_found)/checked): inscription: ({}({})/{}); inscription_id: ({}({})/{}). cost: {:?}",
        result,
        total,
        mismatched_count,
        not_found_count,
        checked_count,
        mismatched_inscription_id_count,
        not_found_inscription_id_count,
        checked_count,
        start_time.elapsed()
    );
}

// clear metadata, because it's not deterministic in genesis cmd
fn clear_metadata(state: &mut ObjectState) {
    state.metadata.state_root = None;
    state.metadata.created_at = 0;
    state.metadata.updated_at = 0;
}

// if mismatched, return true & write output
fn write_mismatched_state_output<T: MoveStructState + std::fmt::Debug, R: std::fmt::Debug>(
    output_writer: &mut BufWriter<File>,
    prefix: &str,
    exp: ObjectState,
    act: Option<ObjectState>,
    src_data: Option<R>, // write source data to output if mismatched for debug
) -> (bool, bool) {
    let mut mismatched = false;
    let mut not_found = false;
    let (act_str, exp_str) = match act {
        Some(act) => {
            let mut act = act;
            clear_metadata(&mut act);
            let exp_decoded: Result<T, _> = T::from_bytes(&exp.value);
            let act_decoded: Result<T, _> = T::from_bytes(&act.value);
            if exp != act {
                mismatched = true;
                (
                    format!("{:?}", act_decoded.unwrap()),
                    format!("{:?}", exp_decoded.unwrap()),
                )
            } else {
                ("".to_string(), "".to_string())
            }
        }
        None => {
            mismatched = true;
            not_found = true;
            let exp_decoded: Result<T, _> = T::from_bytes(&exp.value);
            ("None".to_string(), format!("{:?}", exp_decoded.unwrap()))
        }
    };
    if !mismatched {
        return (false, false);
    }

    writeln!(
        output_writer,
        "{} mismatched: exp: {:?}, act: {:?}, src_data: {:?}",
        prefix, exp_str, act_str, src_data
    )
    .expect("Unable to write line");
    writeln!(output_writer, "--------------------------------").expect("Unable to write line");
    (mismatched, not_found)
}
