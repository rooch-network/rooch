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
