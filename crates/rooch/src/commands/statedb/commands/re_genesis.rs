// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use metrics::RegistryService;
use rooch_config::RoochOpt;
use rooch_config::R_OPT_NET_HELP;
use rooch_db::RoochDB;
use rooch_genesis::RoochGenesis;
use rooch_store::meta_store::MetaStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::BuiltinChainID::Main;
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID, RoochNetwork};
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// Import BTC ordinals & UTXO for genesis
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
    pub startup_backup_path: PathBuf,

    #[clap(long)]
    pub restore: bool,

    #[clap(long)]
    pub output_only: bool,

    #[clap(long)]
    pub restore_genesis: bool,
}

impl ReGenesisCommand {
    pub async fn execute(self) -> RoochResult<()> {
        if self.restore_genesis {
            restore_genesis(self.base_data_dir.clone(), self.chain_id.clone());
            return Ok(());
        }

        if self.output_only {
            write_startup(
                self.base_data_dir.clone(),
                self.chain_id.clone(),
                self.startup_backup_path.clone(),
            );
            return Ok(());
        }

        if self.restore {
            restore_startup(
                self.base_data_dir.clone(),
                self.chain_id.clone(),
                self.startup_backup_path.clone(),
            );
        } else {
            clean_startup(self.base_data_dir.clone(), self.chain_id.clone());
        }

        Ok(())
    }
}

fn restore_genesis(base_data_dir: Option<PathBuf>, chain_id: Option<RoochChainID>) {
    let opt = RoochOpt::new_with_default(base_data_dir.clone(), chain_id.clone(), None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();

    let network: RoochNetwork = Main.into();

    let genesis = RoochGenesis::build(network).unwrap();

    rooch_db
        .moveos_store
        .config_store
        .save_genesis(genesis.genesis_info())
        .unwrap();
    log::info!("Restore genesis info success");
}

fn restore_startup_info() {}

fn write_startup(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
    startup_backup_path: PathBuf,
) {
    let opt = RoochOpt::new_with_default(base_data_dir.clone(), chain_id.clone(), None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();

    let writer = std::fs::File::create(startup_backup_path).unwrap();
    let mut writer = std::io::BufWriter::new(writer);

    let startup_info = rooch_db
        .moveos_store
        .config_store
        .get_startup_info()
        .unwrap();
    if let Some(startup_info) = startup_info {
        writeln!(writer, "{}", serde_json::to_string(&startup_info).unwrap())
            .expect("write failed");
    }

    let origin_genesis_info = rooch_db.moveos_store.config_store.get_genesis().unwrap();

    if let Some(origin_genesis_info) = origin_genesis_info {
        writeln!(
            writer,
            "{}",
            serde_json::to_string(&origin_genesis_info).unwrap()
        )
        .expect("write failed");
    }

    writer.flush().unwrap();
}

fn restore_startup(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
    startup_backup_path: PathBuf,
) {
    let opt = RoochOpt::new_with_default(base_data_dir.clone(), chain_id.clone(), None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();

    let reader = std::fs::File::open(startup_backup_path).unwrap();
    let reader = std::io::BufReader::new(reader);
    let mut lines = reader.lines();
    let network: RoochNetwork = BuiltinChainID::Main.into();

    let genesis = RoochGenesis::build(network).unwrap();
    let startup_info = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();

    rooch_db
        .moveos_store
        .config_store
        .save_genesis(genesis.genesis_info())
        .unwrap();
    rooch_db
        .moveos_store
        .config_store
        .save_startup_info(startup_info)
        .unwrap();

    log::info!("Restore startup info and genesis info success");
}

fn clean_startup(base_data_dir: Option<PathBuf>, chain_id: Option<RoochChainID>) {
    let opt = RoochOpt::new_with_default(base_data_dir.clone(), chain_id.clone(), None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();

    // let origin_startup_info = rooch_db
    //     .moveos_store
    //     .config_store
    //     .get_startup_info()
    //     .unwrap()
    //     .unwrap();
    // let origin_genesis_info = rooch_db
    //     .moveos_store
    //     .config_store
    //     .get_genesis()
    //     .unwrap()
    //     .unwrap();
    //
    // let writer = std::fs::File::create(startup_backup_path).unwrap();
    // let mut writer = std::io::BufWriter::new(writer);
    // writeln!(
    //     writer,
    //     "{}",
    //     serde_json::to_string(&origin_genesis_info).unwrap()
    // )
    // .unwrap();
    // writeln!(
    //     writer,
    //     "{}",
    //     serde_json::to_string(&origin_startup_info).unwrap()
    // )
    // .unwrap();
    // writer.flush().unwrap();

    rooch_db
        .moveos_store
        .config_store
        .delete_startup_info()
        .unwrap();
    rooch_db.moveos_store.config_store.delete_genesis().unwrap();
    println!(
        "{:?}",
        rooch_db
            .rooch_store
            .get_sequencer_info()
            .unwrap()
            .unwrap()
            .to_string()
    );
    rooch_db.rooch_store.clean_sequencer_info().unwrap();

    log::info!("Clean startup info and genesis info success");
}
