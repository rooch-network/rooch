// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{mpsc, Arc};
use std::time::Instant;
use std::{fmt, thread};

use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use serde::{Deserialize, Serialize};

use crate::commands::statedb::commands::genesis_ord::{
    apply_inscription_updates, produce_inscription_updates,
};
use crate::commands::statedb::commands::genesis_utxo::{
    apply_address_updates, apply_utxo_updates, produce_address_map_updates, produce_utxo_updates,
};
use crate::commands::statedb::commands::inscription::InscriptionStats;
use crate::commands::statedb::commands::{init_rooch_db, OutpointInscriptionsMap};

/// Import BTC Inscription & UTXO & rooch_address:BTC_address mapping for genesis
#[derive(Debug, Parser)]
pub struct GenesisCommand {
    #[clap(
        long,
        help = "source data dir. ord: <source_data_dir>/ord, utxo: <source_data_dir>/utxo, ord_stats: <source_data_dir>/ord_stats, outpoint_inscriptions_map: <source_data_dir>/outpoint_inscriptions_map, checksum: <source_data_dir>/checksum"
    )]
    pub source_data_dir: PathBuf,
    #[clap(
        long,
        default_value = "2097152",
        help = "batch size submitted to state db. Set it smaller if memory is limited."
    )]
    pub utxo_batch_size: Option<usize>,
    #[clap(
        long,
        default_value = "1048576",
        help = "batch size submitted to state db. Set it smaller if memory is limited."
    )] // ord may have a large body, so set a smaller batch
    pub ord_batch_size: Option<usize>,
    #[clap(
        long,
        default_value = "all",
        help = "genesis job: inscription, utxo, all"
    )]
    pub job: Option<GenesisJob>,

    #[clap(
        long = "data-dir",
        short = 'd',
        help = "path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name"
    )]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum GenesisJob {
    Inscription = 0b01,
    Utxo = 0b10,
    #[default]
    All = 0b11,
}

impl GenesisJob {
    pub fn contains(self, job: GenesisJob) -> bool {
        (self as u8 & job as u8) != 0
    }
}

impl Display for GenesisJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GenesisJob::Inscription => write!(f, "inscription"),
            GenesisJob::Utxo => write!(f, "utxo"),
            GenesisJob::All => write!(f, "all"),
        }
    }
}

impl FromStr for GenesisJob {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inscription" => Ok(GenesisJob::Inscription),
            "utxo" => Ok(GenesisJob::Utxo),
            "all" => Ok(GenesisJob::All),
            _ => Ok(GenesisJob::All),
        }
    }
}

impl GenesisCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let start_time = Instant::now();

        let ord_source_path = self.source_data_dir.join("ord");
        let ord_exp_stats_path = self.source_data_dir.join("ord_stats");
        let utxo_source_path = self.source_data_dir.join("utxo");
        let outpoint_inscriptions_map_path = self.source_data_dir.join("outpoint_inscriptions_map");

        let outpoint_inscriptions_map = Arc::new(OutpointInscriptionsMap::load_or_index(
            outpoint_inscriptions_map_path.clone(),
            Some(ord_source_path.clone()),
        ));

        self.validate_checksum()?; // validate checksum after outpoint_inscriptions_map dump to avoid not exist error

        // import inscriptions and utxo parallel
        let rooch_db = init_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());
        let moveos_store = rooch_db.moveos_store;
        let moveos_store_arc = Arc::new(moveos_store);
        let mut handles = vec![];

        let job = self.job.unwrap();
        // import inscriptions
        if job.clone().contains(GenesisJob::Inscription) {
            let moveos_store_arc = Arc::clone(&moveos_store_arc);
            let handle = thread::spawn(move || {
                let (ord_tx, ord_rx) = mpsc::sync_channel(2);
                let produce_inscription_updates_thread = thread::spawn(move || {
                    produce_inscription_updates(
                        ord_tx,
                        ord_source_path.clone(),
                        self.ord_batch_size.unwrap(),
                    )
                });
                let apply_inscription_updates_thread = thread::spawn(move || {
                    apply_inscription_updates(
                        ord_rx,
                        moveos_store_arc,
                        InscriptionStats::load_from_file(ord_exp_stats_path.clone()),
                    );
                });
                produce_inscription_updates_thread.join().unwrap();
                apply_inscription_updates_thread.join().unwrap();
            });
            handles.push(handle);
        }

        if job.contains(GenesisJob::Utxo) {
            // import utxo
            let utxo_source_path_arc = Arc::new(utxo_source_path.clone());
            let utxo_batch_size = self.utxo_batch_size.unwrap();
            let moveos_store_arc0 = Arc::clone(&moveos_store_arc);
            let moveos_store_arc1 = Arc::clone(&moveos_store_arc);

            let handle = thread::spawn(move || {
                // produce address map updates
                let (addr_tx, addr_rx) = mpsc::sync_channel(2);
                let utxo_source_path_arc0 = Arc::clone(&utxo_source_path_arc);
                let produce_addr_updates_thread = thread::spawn(move || {
                    produce_address_map_updates(addr_tx, utxo_source_path_arc0, utxo_batch_size)
                });
                // produce utxo updates
                let (utxo_tx, utxo_rx) = mpsc::sync_channel(2);
                let utxo_source_path_arc1 = Arc::clone(&utxo_source_path_arc);
                let produce_utxo_updates_thread = thread::spawn(move || {
                    produce_utxo_updates(
                        utxo_tx,
                        utxo_source_path_arc1,
                        utxo_batch_size,
                        Some(outpoint_inscriptions_map),
                    )
                });
                let apply_addr_updates_thread = thread::spawn(move || {
                    apply_address_updates(addr_rx, moveos_store_arc0);
                });
                let apply_utxo_updates_thread = thread::spawn(move || {
                    apply_utxo_updates(utxo_rx, moveos_store_arc1);
                });

                produce_addr_updates_thread.join().unwrap();
                produce_utxo_updates_thread.join().unwrap();
                apply_addr_updates_thread.join().unwrap();
                apply_utxo_updates_thread.join().unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        println!(
            "genesis inscriptions and utxo imported, cost: {:?}",
            start_time.elapsed()
        );

        Ok(())
    }

    // For checksum file itself protected by sha256, see README for more details
    // For file content protected by xxh3, enough strong for file integrity check
    fn validate_checksum(&self) -> RoochResult<()> {
        let checksums = self.load_checksum();
        let filenames = vec!["ord", "ord_stats", "utxo", "outpoint_inscriptions_map"];
        for filename in filenames {
            self.validate_file_checksum(filename, checksums.clone())?;
        }
        Ok(())
    }

    fn validate_file_checksum(
        &self,
        file_name: &str,
        checksums: HashMap<String, u64>,
    ) -> RoochResult<()> {
        let file_path = self.source_data_dir.join(file_name);
        let file_checksum = calc_file_checksum(&file_path);
        assert_eq!(
            *checksums.get(file_name).unwrap(),
            file_checksum,
            "{} checksum mismatch",
            file_name
        );
        Ok(())
    }

    fn load_checksum(&self) -> HashMap<String, u64> {
        let checksum_path = self.source_data_dir.join("checksum");
        let mut checksum = HashMap::new();
        if checksum_path.exists() {
            let mut file = File::open(checksum_path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    let digest = match u64::from_str_radix(parts[1], 16) {
                        Ok(value) => value,
                        Err(e) => panic!("invalid checksum digest: {}", e),
                    };
                    checksum.insert(parts[0].to_string(), digest);
                } else {
                    panic!("invalid checksum file format");
                }
            }
        }
        checksum
    }
}

fn calc_file_checksum(file_path: &PathBuf) -> u64 {
    let mut file = File::open(file_path).unwrap();
    let mut hasher = xxhash_rust::xxh3::Xxh3::default();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    hasher.digest()
}
