// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::ffi::c_int;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

use clap::Parser;
use criterion::Criterion;
use lazy_static::lazy_static;
use pprof::criterion::{Output, PProfProfiler};
use serde::{Deserialize, Serialize};

use rooch_types::bitcoin::data_import_config::DataImportMode;

lazy_static! {
    static ref BENCH_TX_CONFIG_PATH: String = std::env::var("ROOCH_BENCH_TX_CONFIG_PATH")
        .unwrap_or_else(|_| String::from("config/bench_tx.toml"));
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PProfOutput {
    Proto,
    Flamegraph,
}

impl FromStr for PProfOutput {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proto" => Ok(PProfOutput::Proto),
            "flamegraph" => Ok(PProfOutput::Flamegraph),
            _ => Err(format!("invalid pprof output format: {}", s)),
        }
    }
}

impl Display for PProfOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PProfOutput::Proto => write!(f, "proto"),
            PProfOutput::Flamegraph => write!(f, "flamegraph"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TxType {
    Empty,
    Transfer,
    BtcBlock,
}

impl FromStr for TxType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transfer" => Ok(TxType::Transfer),
            "btc_block" => Ok(TxType::BtcBlock),
            "empty" => Ok(TxType::Empty),
            _ => Err(format!("invalid tx type: {}", s)),
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TxType::Empty => "empty".to_string(),
            TxType::Transfer => "transfer".to_string(),
            TxType::BtcBlock => "btc_blk".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Parser, Eq)]
pub struct BenchTxConfig {
    pub tx_type: Option<TxType>,
    // empty(default)/transfer/btc-block
    pub data_import_mode: Option<DataImportMode>,
    // utxo(default)/ord/none/full
    pub btc_block_dir: Option<String>,
    // btc block dir, file name: <height>.hex
    pub pprof_output: Option<PProfOutput>, // flamegraph(default)/proto
}

impl Default for BenchTxConfig {
    fn default() -> Self {
        Self {
            tx_type: Some(TxType::Empty),
            data_import_mode: Some(DataImportMode::UTXO),
            btc_block_dir: None,
            pprof_output: Some(PProfOutput::Flamegraph),
        }
    }
}

impl BenchTxConfig {
    pub fn adjust(&mut self) {
        self.tx_type.get_or_insert(TxType::Empty);
        self.data_import_mode.get_or_insert(DataImportMode::UTXO);
        // if tx_type is btc_block, btc_block_dir must be existed, if not, panic
        if self.tx_type == Some(TxType::BtcBlock) {
            self.btc_block_dir
                .as_ref()
                .expect("btc_block_dir must be existed");
        }
        self.pprof_output.get_or_insert(PProfOutput::Flamegraph);
    }

    pub fn load() -> Self {
        let path = &*BENCH_TX_CONFIG_PATH;
        let mut config = BenchTxConfig::default();
        match std::fs::read_to_string(path) {
            Ok(config_data) => match toml::from_str::<BenchTxConfig>(&config_data) {
                Ok(mut parsed_config) => {
                    parsed_config.adjust();
                    config = parsed_config;
                }
                Err(e) => {
                    log::error!("Failed to parse config file: {}", e);
                }
            },
            Err(e) => {
                log::error!("Failed to read config file: {}", e);
            }
        };
        config
    }
}

pub struct CriterionConfig {
    pub sample_size: usize,
    pub warm_up_time: Duration,
    pub frequency: c_int,
    pub measurement_time: Duration,
    pub pprof_output: PProfOutput,
}

impl Default for CriterionConfig {
    fn default() -> Self {
        Self {
            sample_size: 500,
            warm_up_time: Duration::from_millis(1), // no need to warm this heavy operation
            frequency: 2000,
            measurement_time: Duration::from_millis(1000),
            pprof_output: PProfOutput::Flamegraph,
        }
    }
}

pub fn configure_criterion(config: Option<CriterionConfig>) -> Criterion {
    let cfg = config.unwrap_or_default();

    let profiler = match cfg.pprof_output {
        PProfOutput::Proto => PProfProfiler::new(cfg.frequency, Output::Protobuf),
        PProfOutput::Flamegraph => PProfProfiler::new(cfg.frequency, Output::Flamegraph(None)),
    };
    Criterion::default()
        .with_profiler(profiler)
        .warm_up_time(cfg.warm_up_time)
        .sample_size(cfg.sample_size)
        .measurement_time(cfg.measurement_time)
}
