// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::statedb::commands::export::ExportCommand;
use crate::commands::statedb::commands::inscription::{
    gen_inscription_id_update, InscriptionSource,
};
use crate::commands::statedb::commands::utxo::{AddressMappingData, UTXORawData};
use crate::commands::statedb::commands::{
    init_job, new_csv_writer, ExportWriter, ExportWriterPreprocessor, OutpointInscriptionsMap,
};
use bitcoin::OutPoint;
use clap::Parser;
use framework_types::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_vm_types::values::Value;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::move_std::option::MoveOption;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::{DynamicField, ObjectID, ObjectMeta};
use moveos_types::state::{
    FieldKey, MoveState, MoveStructState, MoveStructType, MoveType, ObjectState,
};
use moveos_types::state_resolver::{RootObjectResolver, StatelessResolver};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::ord::{Inscription, InscriptionID, InscriptionStore, MODULE_NAME};
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::error::RoochResult;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::{Instant, SystemTime};
use xorf::{BinaryFuse8, Filter};
use xxhash_rust::xxh3::xxh3_64;

/// Verify BTC ordinals & UTXO data in statedb by source data
#[derive(Debug, Parser)]
pub struct GenesisVerifyCommand {
    #[clap(long, short = 'i')]
    /// utxo source data file. like ~/.rooch/local/utxo.csv or utxo.csv
    /// The file format is csv, and the first line is the header, the header is as follows:
    /// count, txid, vout, height, coinbase, amount, script, type,address
    pub utxo_source: PathBuf,
    #[clap(long)]
    /// ord source data file. like ~/.rooch/local/ord or ord, ord_input must be sorted by sequence_number
    /// ord_input & utxo_input must be at the same height
    pub ord_source: Option<PathBuf>,
    #[clap(
        long,
        help = "outpoint(original):inscriptions(original inscription_id) map dump path"
    )]
    pub outpoint_inscriptions_map_dump_path: Option<PathBuf>,
    #[clap(
        long,
        help = "random mode, for randomly select 1/sample_rate inscriptions & utxos to verify"
    )]
    pub random_mode: bool,
    #[clap(
        long,
        help = "sample rate, for randomly select 1/sample_rate inscriptions & utxos to verify. Set 0 if you want to verify cases only",
        default_value = "1000"
    )]
    pub sample_rate: u32,
    #[clap(long, help = "mismatched output path")]
    pub mismatched_output_dir: PathBuf,
    #[clap(
        long,
        help = "Enable this to verify UTXO source data using state database instead of the default behavior of verifying state database using UTXO source data.\
        output: FieldKey,ObjecState,metadata,value"
    )]
    pub reverse: bool,

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
        let outpoint_inscriptions_map = if self.outpoint_inscriptions_map_dump_path.is_some() {
            Some(Arc::new(OutpointInscriptionsMap::load_or_index(
                self.outpoint_inscriptions_map_dump_path.clone().unwrap(),
                self.ord_source.clone(),
            )))
        } else {
            None
        };
        let since_the_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_secs();
        let random_mode = self.random_mode;
        let moveos_store = Arc::new(moveos_store);
        let moveos_store_clone = Arc::clone(&moveos_store);
        // create dir if self.mismatched_output_dir not exists
        std::fs::create_dir_all(&self.mismatched_output_dir)?;

        if self.reverse {
            verify_reverse(
                self.utxo_source,
                create_output_path(&self.mismatched_output_dir, "utxo_extra", timestamp),
                create_output_path(&self.mismatched_output_dir, "addr_extra", timestamp),
                moveos_store,
                root.clone().state_root.unwrap(),
                outpoint_inscriptions_map,
            )?;
            return Ok(());
        }

        // verify inscriptions
        let ord_mismatched_output =
            create_output_path(&self.mismatched_output_dir, "ord_mismatched", timestamp);
        let ord_new_cases_output =
            create_output_path(&self.mismatched_output_dir, "ord_cases", timestamp);
        let ord_cases = find_latest_file_with_prefix(&self.mismatched_output_dir, "ord_cases");
        let inscription_source_path = self.ord_source.clone();
        let root_clone_0 = root.clone();
        let verify_inscription_thread = thread::Builder::new()
            .name("verify-inscription".to_string())
            .spawn(move || {
                verify_inscription(
                    inscription_source_path,
                    ord_cases,
                    ord_new_cases_output,
                    moveos_store_clone,
                    root_clone_0,
                    random_mode,
                    self.sample_rate,
                    ord_mismatched_output,
                );
            })?;
        // verify utxo
        let utxo_mismatched_output =
            create_output_path(&self.mismatched_output_dir, "utxo_mismatched", timestamp);
        let utxo_new_cases_output =
            create_output_path(&self.mismatched_output_dir, "utxo_cases", timestamp);
        let utxo_cases = find_latest_file_with_prefix(&self.mismatched_output_dir, "utxo_cases");
        let moveos_store_clone = Arc::clone(&moveos_store);
        let verify_utxo_thread = thread::Builder::new()
            .name("verify-utxo".to_string())
            .spawn(move || {
                verify_utxo(
                    self.utxo_source,
                    utxo_cases,
                    utxo_new_cases_output,
                    moveos_store_clone,
                    root.clone(),
                    outpoint_inscriptions_map,
                    random_mode,
                    self.sample_rate,
                    utxo_mismatched_output,
                );
            })?;

        verify_inscription_thread.join().unwrap();
        verify_utxo_thread.join().unwrap();

        println!(
            "genesis verify done, output with timestamp: {:?} in {:?}, cost: {:?}",
            timestamp,
            self.mismatched_output_dir,
            start_time.elapsed()
        );

        Ok(())
    }
}

struct UTXOFilter {
    outpoint_filter: BinaryFuse8,
    utxo_writer: csv::Writer<File>,

    addr_filter: BinaryFuse8,
    addr_writer: csv::Writer<File>,
}

impl UTXOFilter {
    fn new(
        input: PathBuf,
        utxo_output: PathBuf,
        addr_output: PathBuf,
        outpoint_inscriptions_map: Option<Arc<OutpointInscriptionsMap>>,
    ) -> UTXOFilter {
        let start_time = Instant::now();
        tracing::info!("start building utxo & address filter");

        let mut reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
        let mut is_title_line = true;
        let mut utxo_keys = Vec::with_capacity(200_000_000);
        let mut addr_keys = Vec::with_capacity(80_000_000);
        let mut added_address_set: FxHashSet<String> =
            FxHashSet::with_capacity_and_hasher(60_000_000, Default::default());

        for line in reader.by_ref().lines() {
            let line = line.unwrap();
            if is_title_line {
                is_title_line = false;
                if line.starts_with("count") {
                    continue;
                }
            }

            let mut utxo_raw = UTXORawData::from_str(&line);
            let (utxo_key, _) = utxo_raw.gen_utxo_update(outpoint_inscriptions_map.clone());
            utxo_keys.push(xxh3_64(&utxo_key.0));

            let (_, exp_addr_map) = utxo_raw.gen_address_mapping_data();
            let addr_updates = if let Some(address_mapping_data) = exp_addr_map {
                address_mapping_data.gen_update(&mut added_address_set)
            } else {
                None
            };
            if addr_updates.is_some() {
                let (addr_key, _) = addr_updates.unwrap();
                addr_keys.push(xxh3_64(&addr_key.0));
            }
        }
        let outpoint_filter = BinaryFuse8::try_from(&utxo_keys).unwrap();
        let addr_filter = BinaryFuse8::try_from(&addr_keys).unwrap();

        let utxo_writer = new_csv_writer(utxo_output);
        let addr_writer = new_csv_writer(addr_output);

        println!("filtering done, cost: {:?}", start_time.elapsed());

        Self {
            outpoint_filter,
            addr_filter,
            utxo_writer,
            addr_writer,
        }
    }

    fn contains(&self, field_key: &FieldKey, is_utxo: bool) -> bool {
        if is_utxo {
            self.outpoint_filter.contains(&xxh3_64(&field_key.0))
        } else {
            self.addr_filter.contains(&xxh3_64(&field_key.0))
        }
    }
}

impl ExportWriterPreprocessor for UTXOFilter {
    fn process(&mut self, k: &FieldKey, v: &ObjectState) -> (FieldKey, ObjectState) {
        let type_tag = v.metadata.object_type.clone();
        let is_utxo = type_tag == UTXO::type_tag();
        if !self.contains(k, is_utxo) {
            let (writer, value_str) = if is_utxo {
                (
                    &mut self.utxo_writer,
                    format!("{:?}", UTXO::from_bytes(v.value.clone()).unwrap()),
                )
            } else {
                (
                    &mut self.addr_writer,
                    format_object_value::<DynamicField<AccountAddress, BitcoinAddress>>(
                        v.value.clone(),
                    ),
                )
            };

            writer
                .write_record(&[
                    k.to_string(),
                    v.to_string(),
                    format!("{:?}", v.metadata.clone()),
                    value_str,
                ])
                .unwrap();
        }

        (*k, v.clone())
    }

    fn flush(&mut self) {
        self.utxo_writer.flush().unwrap();
        self.addr_writer.flush().unwrap()
    }
}

fn format_object_value<T: MoveStructState + std::fmt::Debug>(value: Vec<u8>) -> String {
    format!("{:?}", T::from_bytes(value).unwrap())
}

fn verify_reverse(
    input: PathBuf,
    utxo_output: PathBuf,
    addr_output: PathBuf,
    moveos_store_arc: Arc<MoveOSStore>,
    root_state_root: H256,
    outpoint_inscriptions_map: Option<Arc<OutpointInscriptionsMap>>,
) -> anyhow::Result<()> {
    let filter = UTXOFilter::new(input, utxo_output, addr_output, outpoint_inscriptions_map);

    let mut writer = ExportWriter::new(None, Some(Box::new(filter)));

    ExportCommand::export_object(
        &moveos_store_arc,
        root_state_root,
        RoochToBitcoinAddressMapping::object_id(),
        &mut writer,
    )?;
    writer.flush()?;
    ExportCommand::export_object(
        &moveos_store_arc,
        root_state_root,
        BitcoinUTXOStore::object_id(),
        &mut writer,
    )?;

    writer.flush()?;
    Ok(())
}

fn verify_utxo(
    input: PathBuf,
    case_path: Option<PathBuf>,
    case_new_path: PathBuf,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    outpoint_inscriptions_map: Option<Arc<OutpointInscriptionsMap>>,
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
    let mut new_cases = UTXOCases {
        cases: HashSet::new(),
    };

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

        let raw_output = OutPoint {
            txid: utxo_raw.txid,
            vout: utxo_raw.vout,
        };

        let need_verify = if !random_mode {
            true
        } else {
            let is_case = cases.contains(&raw_output);
            if sample_rate == 0 {
                is_case
            } else {
                rand::random::<u32>() % sample_rate == 0 || is_case
            }
        };

        if !need_verify {
            continue;
        }

        // check utxo
        utxo_checked_count += 1;
        let (exp_utxo_key, exp_utxo_state) =
            utxo_raw.gen_utxo_update(outpoint_inscriptions_map.clone());
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
            new_cases.insert(raw_output);
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
                new_cases.insert(raw_output);
                address_mismatched_count += 1;
            }
            if not_found {
                address_not_found_count += 1;
            }
        }
    }
    output_writer.flush().expect("Unable to flush writer");
    new_cases.dump(case_new_path);

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
    input: Option<PathBuf>,
    case_path: Option<PathBuf>,
    case_new_path: PathBuf,
    moveos_store_arc: Arc<MoveOSStore>,
    root: ObjectMeta,
    random_mode: bool,
    sample_rate: u32,
    mismatched_output: PathBuf,
) {
    if input.is_none() {
        return;
    }
    let input = input.unwrap();
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
    let mut new_cases = OrdCases {
        cases: HashSet::new(),
    };

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

        let need_verify = if !random_mode {
            true
        } else {
            let is_case = cases.contains(source.sequence_number);
            if sample_rate == 0 {
                is_case
            } else {
                rand::random::<u32>() % sample_rate == 0 || is_case
            }
        };

        if !need_verify {
            continue;
        }
        // check inscription
        checked_count += 1;
        let (exp_key, exp_state, exp_inscription_id) = source.gen_update();
        let act_inscription_state = resolver
            .get_field_at(inscription_store_state_root, &exp_key)
            .unwrap();
        let exp_inscription = Inscription::from_bytes(exp_state.value).unwrap();
        let exp_inscription_cmp: InscriptionForComparison =
            InscriptionForComparison::from(&exp_inscription);
        let exp_state_cmp = ObjectState {
            metadata: exp_state.metadata,
            value: exp_inscription_cmp.to_bytes(),
        };
        let act_state_cmp = act_inscription_state.clone().map(|state| {
            let act_inscription = Inscription::from_bytes(state.value).unwrap();
            let act_inscription_cmp: InscriptionForComparison =
                InscriptionForComparison::from(&act_inscription);
            ObjectState {
                metadata: state.metadata,
                value: act_inscription_cmp.to_bytes(),
            }
        });

        let (mismatched, not_found) =
            write_mismatched_state_output::<InscriptionForComparison, InscriptionSource>(
                &mut output_writer,
                "[inscription]",
                exp_state_cmp,
                act_state_cmp,
                Some(source.clone()),
            );
        if mismatched {
            mismatched_count += 1;
            new_cases.insert(source.sequence_number);
        }
        if not_found {
            not_found_count += 1;
        }
        // check inscription_id
        let (exp_inscription_id_key, exp_inscription_id_state) =
            gen_inscription_id_update(total - 1, exp_inscription_id);
        let act_inscription_id_state = resolver
            .get_field_at(inscription_store_state_root, &exp_inscription_id_key)
            .unwrap();
        let (mismatched, not_found) =
            write_mismatched_state_output::<DynamicField<u32, InscriptionID>, InscriptionID>(
                &mut output_writer,
                "[inscription_id]",
                exp_inscription_id_state,
                act_inscription_id_state.clone(),
                Some(source.id),
            );
        if mismatched {
            mismatched_inscription_id_count += 1;
            new_cases.insert(source.sequence_number);
        }
        if not_found {
            not_found_inscription_id_count += 1;
        }
    }

    output_writer.flush().expect("Unable to flush writer");
    new_cases.dump(case_new_path);

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

// clear metadata for comparison
fn clear_metadata(state: &mut ObjectState) {
    // which are not deterministic in genesis cmd
    state.metadata.state_root = None;
    state.metadata.created_at = 0;
    state.metadata.updated_at = 0;
}

// if mismatched, return true & write output
fn write_mismatched_state_output<
    T: MoveStructState + std::fmt::Debug + 'static,
    R: std::fmt::Debug,
>(
    output_writer: &mut BufWriter<File>,
    prefix: &str,
    exp: ObjectState,
    act: Option<ObjectState>,
    src_data: Option<R>, // write source data to output if mismatched for debug
) -> (bool, bool) {
    // mismatched, not_found
    let mut mismatched_meta = false;
    let mut mismatched_val = false;
    let mut mismatched_obj = false;
    let not_found = act.is_none();

    let exp_decoded = T::from_bytes(&exp.value).unwrap();

    let (diff_meta, diff_val, exp_meta) = match act {
        Some(act) => {
            let mut diff_meta = "".to_string();
            let mut diff_val = "".to_string();
            let mut act = act;
            clear_metadata(&mut act);
            mismatched_meta = exp.metadata != act.metadata;
            mismatched_val = exp.value != act.value;
            mismatched_obj = mismatched_meta && mismatched_val;

            if mismatched_meta {
                diff_meta = compare_and_format::<ObjectMeta>(&exp.metadata, &act.metadata);
            }

            if mismatched_val {
                let act_decoded = T::from_bytes(&act.value).unwrap();
                diff_val = compare_and_format::<T>(&exp_decoded, &act_decoded);
            }
            (diff_meta, diff_val, exp.metadata)
        }
        None => ("".to_string(), "".to_string(), exp.metadata),
    };

    let mismatched = mismatched_obj || not_found || mismatched_meta || mismatched_val;

    if !mismatched {
        return (false, false);
    }

    let state = if not_found {
        "not_found".to_string()
    } else if mismatched_meta {
        "mismatched_meta".to_string()
    } else if mismatched_val {
        "mismatched_val".to_string()
    } else {
        "mismatched_obj".to_string()
    };
    if not_found {
        writeln!(
            output_writer,
            "{} {}. exp_meta: {:?}; exp_val: {:?}; src_data: {:?}",
            prefix, state, exp_meta, exp_decoded, src_data
        )
        .expect("Unable to write line");
    }
    if mismatched_obj {
        writeln!(
            output_writer,
            "{} {}. <diff_meta: {:?}>; <diff_val: {:?}>. exp_meta: {:?}; exp_val: {:?}; src_data: {:?}",
            prefix, state, diff_meta, diff_val, exp_meta, exp_decoded,src_data
        )
        .expect("Unable to write line");
    } else if mismatched_meta {
        writeln!(
            output_writer,
            "{} {}. <diff_meta: {:?}>. exp_meta: {:?}; exp_val: {:?}; src_data: {:?}",
            prefix, state, diff_meta, exp_meta, exp_decoded, src_data
        )
        .expect("Unable to write line");
    } else {
        writeln!(
            output_writer,
            "{} {}. <diff_val: {:?}>. exp_meta: {:?}, exp_val: {:?}, src_data: {:?}",
            prefix, state, diff_val, exp_meta, exp_decoded, src_data
        )
        .expect("Unable to write line");
    }
    writeln!(output_writer, "--------------------------------").expect("Unable to write line");
    (mismatched, not_found)
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
    fn insert(&mut self, outpoint: OutPoint) {
        self.cases.insert(outpoint);
    }
    fn contains(&self, outpoint: &OutPoint) -> bool {
        self.cases.contains(outpoint)
    }
    fn dump(&self, path: PathBuf) {
        let file = File::create(path).expect("Unable to create utxo cases file");
        let mut writer = BufWriter::new(file);
        for outpoint in &self.cases {
            writeln!(writer, "{}", outpoint).expect("Unable to write line");
        }
        writer.flush().expect("Unable to flush writer");
    }
}

struct OrdCases {
    cases: HashSet<u32>, // sequence_number for easy generating
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
    fn insert(&mut self, sequence_number: u32) {
        self.cases.insert(sequence_number);
    }
    fn contains(&self, sequence_number: u32) -> bool {
        self.cases.contains(&sequence_number)
    }

    fn dump(&self, path: PathBuf) {
        let file = File::create(path).expect("Unable to create ord cases file");
        let mut writer = BufWriter::new(file);
        for ord in &self.cases {
            writeln!(writer, "{}", ord).expect("Unable to write line");
        }
        writer.flush().expect("Unable to flush writer");
    }
}

fn create_output_path(output_dir: &Path, prefix: &str, timestamp: u64) -> PathBuf {
    let file_name = format!("{}_{:?}", prefix, timestamp);
    output_dir.join(file_name)
}

fn find_latest_file_with_prefix(output_dir: &PathBuf, prefix: &str) -> Option<PathBuf> {
    let mut max_timestamp = None;
    let mut max_path = None;
    for entry in std::fs::read_dir(output_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name()?.to_str()?;
            if !filename.starts_with(prefix) {
                continue;
            };
            // get timestamp from file name: <prefix>_<timestamp>
            let timestamp = filename.split('_').last()?.parse::<u64>().unwrap();
            if max_timestamp.is_none() || timestamp > max_timestamp? {
                max_timestamp = Some(timestamp);
                max_path = Some(path);
            }
        }
    }
    max_path
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct InscriptionForComparison {
    pub txid: AccountAddress,
    pub index: u32,
    pub offset: u64,
    pub sequence_number: u32,
    pub inscription_number: u32,
    pub is_curse: bool,
    pub body: Vec<u8>,
    pub content_encoding: MoveOption<MoveString>,
    pub content_type: MoveOption<MoveString>,
    pub metadata: Vec<u8>,
    pub metaprotocol: MoveOption<MoveString>,
    pub parents: Vec<InscriptionID>,
    pub pointer: MoveOption<u64>,
    pub rune: MoveOption<Vec<u8>>, // Changed to Vec<u8> for comparison
}

impl From<&Inscription> for InscriptionForComparison {
    fn from(ins: &Inscription) -> Self {
        InscriptionForComparison {
            txid: ins.id.txid,
            index: ins.id.index,
            offset: ins.location.offset,
            sequence_number: ins.sequence_number,
            inscription_number: ins.inscription_number,
            is_curse: ins.is_cursed,
            body: ins.body.clone(),
            content_encoding: ins.content_encoding.clone(),
            content_type: ins.content_type.clone(),
            metadata: ins.metadata.clone(),
            metaprotocol: ins.metaprotocol.clone(),
            parents: ins.parents.clone().into_iter().map(Into::into).collect(),
            pointer: ins.pointer.clone(),
            rune: MoveOption::from(to_commitment(ins.rune.clone().into())),
        }
    }
}

fn to_commitment(rune: Option<u128>) -> Option<Vec<u8>> {
    rune?;
    let rune = rune?;
    let bytes = rune.to_le_bytes();

    let mut end = bytes.len();

    while end > 0 && bytes[end - 1] == 0 {
        end -= 1;
    }

    Some(bytes[..end].into())
}

fn from_commitment(rune_bytes: Vec<u8>) -> u128 {
    let mut arr = [0u8; 16];
    for (place, element) in arr.iter_mut().zip(rune_bytes.iter()) {
        *place = *element;
    }
    u128::from_le_bytes(arr)
}

impl MoveStructType for InscriptionForComparison {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Inscription");
}

impl MoveStructState for InscriptionForComparison {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
            u64::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
            bool::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            Vec::<ObjectID>::type_layout(),
            MoveOption::<u64>::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
        ])
    }
}

fn compare_and_format<T: Serialize + std::fmt::Debug>(exp: &T, act: &T) -> String {
    let exp_json =
        serde_json::to_value(exp).unwrap_or_else(|_| panic!("Unable to serialize exp: {:?}", exp));
    let act_json =
        serde_json::to_value(act).unwrap_or_else(|_| panic!("Unable to serialize act: {:?}", act));

    let exp_map = exp_json.as_object().unwrap();
    let act_map = act_json.as_object().unwrap();

    let mut keys: HashSet<String> = exp_map.keys().cloned().collect();
    keys.extend(act_map.keys().cloned());

    let mut differences = HashMap::new();

    for key in keys {
        let exp_value = exp_map.get(&key).unwrap_or(&serde_json::Value::Null);
        let act_value = act_map.get(&key).unwrap_or(&serde_json::Value::Null);

        if exp_value != act_value {
            if key == "rune" {
                let exp_rune = rune_from_json_value(exp_value);
                let act_rune = rune_from_json_value(act_value);
                differences.insert(key.clone(), format!("{:?} -> {:?}", exp_rune, act_rune));
            } else {
                differences.insert(key.clone(), format!("{:?} -> {:?}", exp_value, act_value));
            }
        }
    }

    format_differences(differences)
}

fn rune_from_json_value(value: &serde_json::Value) -> Option<u128> {
    match value {
        serde_json::Value::Object(obj) => {
            // Object {"vec": Array [Array [Number(210), Number(2), Number(150), Number(73)]]}
            let arr = obj.get("vec")?.as_array()?;
            let rune_vec = arr.first()?.as_array()?;
            let rune_bytes: Vec<u8> = rune_vec.iter().map(|v| v.as_u64().unwrap() as u8).collect();
            Some(from_commitment(rune_bytes))
        }
        _ => None,
    }
}

fn format_differences(differences: HashMap<String, String>) -> String {
    let mut formatted = String::new();

    for (key, diff) in differences {
        if !formatted.is_empty() {
            formatted.push(';');
        }
        // grep diff_* to find differences
        formatted.push_str(&format!("diff_{}: {}", key, diff));
    }

    formatted
}

#[cfg(test)]
mod tests {
    use crate::commands::statedb::commands::genesis_verify::{
        compare_and_format, rune_from_json_value, to_commitment, InscriptionForComparison,
    };
    use crate::commands::statedb::commands::inscription::InscriptionSource;
    use bitcoin::Txid;
    use moveos_types::move_std::option::MoveOption;
    use rooch_types::bitcoin::ord::InscriptionID;
    use rooch_types::into_address::IntoAddress;
    use std::str::FromStr;

    #[test]
    fn test_rune_from_json_value() {
        let rune = 449272580684223630017036914u128;
        let rune_move_opt = MoveOption::from(to_commitment(Some(rune)));
        let value = serde_json::to_value(rune_move_opt).unwrap();
        let rune_act = rune_from_json_value(&value);
        assert_eq!(rune_act, Some(rune));

        let ins_source = InscriptionSource {
            sequence_number: 74311915,
            inscription_number: 73839872,
            id: InscriptionID {
                txid: Txid::from_str(
                    "ab324803e78a978872b5a71b4838644c5fc0dbb0ffeb4e93c73462854d54d427",
                )
                .unwrap()
                .into_address(),
                index: 0,
            },
            satpoint_outpoint: "ab324803e78a978872b5a71b4838644c5fc0dbb0ffeb4e93c73462854d54d427:1"
                .to_string(),
            satpoint_offset: 0,
            body: None,
            content_encoding: None,
            content_type: None,
            metadata: None,
            metaprotocol: None,
            parent: None,
            pointer: None,
            address: "bc1pl6xn9vvpqna0grwd0tuwyj9z8s55gw8wvrd8885yra9hc9hlkh5s0rcxm2".to_string(),
            rune: Some(449272580684223630017036914),
            charms: 0,
        };

        let ins = ins_source.to_inscription();
        let ins_cmp = InscriptionForComparison::from(&ins);

        let mut ins_cmp_2 = ins_cmp.clone();
        ins_cmp_2.rune = MoveOption::none();
        let diff = compare_and_format(&ins_cmp, &ins_cmp_2);
        assert_eq!(diff, "diff_rune: Some(449272580684223630017036914) -> None");
    }
}
