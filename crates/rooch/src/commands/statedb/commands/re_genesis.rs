// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::statedb::commands::init_rooch_db;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_store::meta_store::MetaStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReGenesisMode {
    #[default]
    Export, // dump GensisInfo, StartupInfo, SequencerInfo
    Remove,  // remove GensisInfo, StartupInfo, SequencerInfo, indexer directory
    Restore, // restore GensisInfo, StartupInfo, SequencerInfo
}

impl Display for ReGenesisMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReGenesisMode::Export => write!(f, "export"),
            ReGenesisMode::Remove => write!(f, "remove"),
            ReGenesisMode::Restore => write!(f, "restore"),
        }
    }
}

impl FromStr for ReGenesisMode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "export" => Ok(ReGenesisMode::Export),
            "remove" => Ok(ReGenesisMode::Remove),
            "restore" => Ok(ReGenesisMode::Restore),
            _ => Err("re-genesis-mode no match"),
        }
    }
}

/// Toolkits for modifying genesis and startup info, for development and testing start rooch server with certain genesis.
#[derive(Debug, Parser)]
pub struct ReGenesisCommand {
    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long)]
    pub export_path: PathBuf,
    #[clap(long)]
    pub mode: Option<ReGenesisMode>,
}

impl ReGenesisCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mode = self.mode.unwrap_or_default();
        match mode {
            ReGenesisMode::Export => self.export(),
            ReGenesisMode::Remove => self.remove(),
            ReGenesisMode::Restore => self.restore(),
        }

        Ok(())
    }

    fn export(&self) {
        let rooch_db = init_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());

        let writer = std::fs::File::create(self.export_path.clone()).unwrap();
        let mut writer = std::io::BufWriter::new(writer);

        let mut outputs = Vec::new();
        let mut outputed = Vec::new();

        let genesis_info = rooch_db.moveos_store.config_store.get_genesis().unwrap();
        if let Some(genesis_info) = genesis_info {
            outputs.push(serde_json::to_string(&genesis_info).unwrap());
            outputed.push("genesis_info");
        }

        let startup_info = rooch_db
            .moveos_store
            .config_store
            .get_startup_info()
            .unwrap();
        if let Some(startup_info) = startup_info {
            outputs.push(serde_json::to_string(&startup_info).unwrap());
            outputed.push("startup_info");
        }

        let sequencer_info = rooch_db.rooch_store.get_sequencer_info().unwrap();
        if let Some(sequencer_info) = sequencer_info {
            outputs.push(serde_json::to_string(&sequencer_info).unwrap());
            outputed.push("sequencer_info");
        }

        for output in outputs {
            writeln!(writer, "{}", output).expect("write failed");
        }
        writer.flush().unwrap();

        log::info!("Export {} success", outputed.join(", "));
    }

    fn remove(&self) {
        let rooch_db = init_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());

        rooch_db.moveos_store.config_store.remove_genesis().unwrap();
        rooch_db
            .moveos_store
            .config_store
            .remove_startup_info()
            .unwrap();
        rooch_db.rooch_store.remove_sequencer_info().unwrap();

        // remove indexer directory:
        // base_data_dir/chain_network_name/rooch/indexer
        let base_data_dir = self.base_data_dir.clone().unwrap();
        let chain_network_name = self.chain_id.clone().unwrap().to_string();
        let indexer_dir = base_data_dir
            .join(chain_network_name)
            .join("roochdb/indexer");
        std::fs::remove_dir_all(indexer_dir).unwrap();
        log::info!("Remove genesis info, startup info and sequencer info success");
    }

    fn restore(&self) {
        let reader = std::fs::File::open(self.export_path.clone()).unwrap();
        let reader = std::io::BufReader::new(reader);
        let mut lines = reader.lines();
        let genesis_info = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        let startup_info = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        let sequencer_info = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();

        let rooch_db = init_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());

        rooch_db
            .moveos_store
            .config_store
            .save_genesis(genesis_info)
            .unwrap();
        rooch_db
            .moveos_store
            .config_store
            .save_startup_info(startup_info)
            .unwrap();
        rooch_db
            .rooch_store
            .save_sequencer_info(sequencer_info)
            .unwrap();

        log::info!("Restore genesis info, startup info and sequencer info success");
    }
}
