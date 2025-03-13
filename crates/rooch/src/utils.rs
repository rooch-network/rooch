// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use itertools::Itertools;
use metrics::RegistryService;
use moveos_config::store_config::RocksdbConfig;
use moveos_types::moveos_std::object::ObjectMeta;
use raw_store::rocks::RocksDB;
use rocksdb::{ColumnFamilyDescriptor, DB};
use rooch_config::da_config::derive_namespace_from_genesis;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_genesis::load_genesis_from_binary;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::address::RoochAddress;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use std::{collections::BTreeMap, str::FromStr};

/// Error message for parsing a map
const PARSE_MAP_SYNTAX_MSG: &str = "Invalid syntax for map. Example: Name=Value,Name2=Value";

/// Parses an inline map of values
///
/// Example: Name=Value,Name2=Value
pub fn parse_map<K: FromStr + Ord, V: FromStr>(str: &str) -> anyhow::Result<BTreeMap<K, V>>
where
    K::Err: 'static + std::error::Error + Send + Sync,
    V::Err: 'static + std::error::Error + Send + Sync,
{
    let mut map = BTreeMap::new();

    // Split pairs by commas
    for pair in str.split_terminator(',') {
        // Split pairs by = then trim off any spacing
        let (first, second): (&str, &str) = pair
            .split_terminator('=')
            .collect_tuple()
            .ok_or_else(|| anyhow::Error::msg(PARSE_MAP_SYNTAX_MSG))?;
        let first = first.trim();
        let second = second.trim();
        if first.is_empty() || second.is_empty() {
            return Err(anyhow::Error::msg(PARSE_MAP_SYNTAX_MSG));
        }

        // At this point, we just give error messages appropriate to parsing
        let key: K = K::from_str(first)?;
        let value: V = V::from_str(second)?;
        map.insert(key, value);
    }
    Ok(map)
}

//#[macro_export]
//macro_rules! sign_and_execute {
//    ($tx_data:expr, $context:expr) => {{
//        let transaction = $context
//            .get_config()
//            .await?
//            .keystore
//            .sign_transaction(&$tx_data.sender, $tx_data)
//            .map_err(|e| RoochError::SignMessageError(e.to_string()))?;
//
//        let client = $context.get_client().await?;
//
//        client
//            .execute_tx(transaction)
//            .await
//            .map_err(|e| RoochError::TransactionError(e.to_string()))
//    }};
//}

pub fn read_line() -> Result<String, anyhow::Error> {
    let mut s = String::new();
    let _ = stdout().flush();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim_end().to_string())
}

pub fn prompt_yes_no(question: &str) -> bool {
    loop {
        println!("{} [yes/no] > ", question);

        let Ok(input) = read_line() else {
            println!("Please answer yes or no.");
            continue;
        };

        match input.trim_start().to_lowercase().as_str() {
            "yes" | "y" => return true,
            "no" | "n" => return false,
            _ => println!("Please answer yes or no."),
        }
    }
}

pub fn get_sequencer_keypair(
    context_options: WalletContextOptions,
    sequencer_account: Option<String>,
) -> RoochResult<RoochKeyPair> {
    let context = context_options.build_require_password()?;
    let sequencer_account = if sequencer_account.is_none() {
        let active_address_opt = context.client_config.active_address;
        if active_address_opt.is_none() {
            return Err(RoochError::ActiveAddressDoesNotExistError);
        }
        active_address_opt.unwrap()
    } else {
        RoochAddress::from_str(sequencer_account.clone().unwrap().as_str()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid sequencer account address: {}", e))
        })?
    };
    context
        .keystore
        .get_key_pair(&sequencer_account, context.get_password())
        .map_err(|e| RoochError::SequencerKeyPairDoesNotExistError(e.to_string()))
}

pub fn open_rooch_db(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (ObjectMeta, RoochDB, SystemTime) {
    let start_time = SystemTime::now();

    let opt = RoochOpt::new_with_default(base_data_dir, chain_id, None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();
    let root = rooch_db.latest_root().unwrap().unwrap();
    (root, rooch_db, start_time)
}

pub fn open_inner_rocks(
    path: &str,
    column_families: Vec<String>,
    readonly: bool,
) -> anyhow::Result<DB> {
    let config = RocksdbConfig::default();
    let mut rocksdb_opts = RocksDB::gen_rocksdb_options(&config);
    let table_opts = RocksDB::generate_table_opts(&config);
    if readonly {
        let error_if_log_file_exists = false;
        let inner = DB::open_cf_for_read_only(
            &rocksdb_opts,
            path,
            column_families,
            error_if_log_file_exists,
        )?;
        Ok(inner)
    } else {
        rocksdb_opts.create_if_missing(true);
        rocksdb_opts.create_missing_column_families(true);
        let inner = DB::open_cf_descriptors(
            &rocksdb_opts,
            path,
            column_families.iter().map(|cf_name| {
                let cf_opts = RocksDB::generate_cf_options(cf_name, &table_opts);
                ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts)
            }),
        )?;
        Ok(inner)
    }
}

pub fn derive_builtin_genesis_namespace_from_rooch_chain_id(
    chain_id: Option<RoochChainID>,
) -> anyhow::Result<Option<String>> {
    if chain_id.is_none() {
        return Ok(None);
    }

    match chain_id.unwrap() {
        RoochChainID::Builtin(builtin_chain_id) => {
            let namespace = derive_builtin_genesis_namespace(builtin_chain_id)?;
            Ok(Some(namespace))
        }
        RoochChainID::Custom(_) => Ok(None),
    }
}

pub fn derive_builtin_genesis_namespace(chain_id: BuiltinChainID) -> anyhow::Result<String> {
    let genesis = load_genesis_from_binary(chain_id)?.expect("Genesis not found");
    let genesis_hash = genesis.genesis_hash();
    Ok(derive_namespace_from_genesis(genesis_hash))
}

pub struct TxSizeHist {
    hist: hdrhistogram::Histogram<u64>,
    tops: BinaryHeap<Reverse<(u64, u64)>>, // (size, tx_order) Use Reverse to keep the smallest element at the top
    top_n: usize,
    title: String,
}

impl TxSizeHist {
    pub fn new(
        title: String,
        top_n: usize,
        low_size: Option<u64>,
        high_size: Option<u64>,
    ) -> anyhow::Result<Self> {
        let low = low_size.unwrap_or(1);
        let high = high_size.unwrap_or(4_096_000);

        Ok(TxSizeHist {
            hist: hdrhistogram::Histogram::<u64>::new_with_bounds(low, high, 3)?,
            tops: BinaryHeap::new(),
            top_n,
            title,
        })
    }

    pub fn record(&mut self, tx_order: u64, size: u64) -> anyhow::Result<()> {
        self.hist.record(size)?;

        if self.tops.len() < self.top_n {
            // Add the new item directly if space is available
            self.tops.push(Reverse((size, tx_order)));
        } else if let Some(&Reverse((smallest_size, _))) = self.tops.peek() {
            // Compare with the smallest item in the heap
            if size > smallest_size {
                self.tops.pop(); // Remove the smallest
                self.tops.push(Reverse((size, tx_order))); // Add the new larger item
            }
        }
        // Keep only top-N
        Ok(())
    }

    /// Returns the top N items, sorted by `tx_size` in descending order
    pub fn get_top(&self) -> Vec<(u64, u64)> {
        let mut sorted: Vec<_> = self.tops.iter().map(|&Reverse(x)| x).collect();
        sorted.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by tx_size in descending order
        sorted
    }

    pub fn print(&mut self) {
        let hist = &self.hist;

        let min_size = hist.min();
        let max_size = hist.max();
        let mean_size = hist.mean();

        println!(
            "-----------------{} Size Stats-----------------",
            self.title
        );
        println!(
            "Percentiles distribution(count: {}): min={}, max={}, mean={:.2}, stdev={:.2}: ",
            hist.len(),
            min_size,
            max_size,
            mean_size,
            hist.stdev()
        );
        let percentiles = [
            1.00, 5.00, 10.00, 20.00, 30.00, 40.00, 50.00, 60.00, 70.00, 80.00, 90.00, 95.00,
            99.00, 99.50, 99.90, 99.95, 99.99,
        ];
        for &p in &percentiles {
            let v = hist.value_at_percentile(p);
            println!("| {:6.2}th=[{}]", p, v);
        }

        // each pair one line
        println!("-------------Top{} transactions--------------", self.top_n);
        let tops = self.get_top();
        for (tx_size, tx_order) in &tops {
            println!("tx_order: {}, size: {}", tx_order, tx_size);
        }
    }
}
