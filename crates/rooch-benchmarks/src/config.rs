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
    TransferLargeObject,
    BtcBlock,
    BtcTx,
}

impl FromStr for TxType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transfer" => Ok(TxType::Transfer),
            "btc_block" => Ok(TxType::BtcBlock),
            "btc_tx" => Ok(TxType::BtcTx),
            "empty" => Ok(TxType::Empty),
            "transfer_large_object" => Ok(TxType::TransferLargeObject),
            _ => Err(format!("invalid tx type: {}", s)),
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TxType::Empty => "empty".to_string(),
            TxType::Transfer => "transfer".to_string(),
            TxType::BtcBlock => "btc_block".to_string(),
            TxType::BtcTx => "btc_tx".to_string(),
            TxType::TransferLargeObject => "transfer_large_object".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Parser, Eq)]
pub struct BenchTxConfig {
    pub tx_type: Option<TxType>,       // empty(default)/transfer/btc-block
    pub btc_block_dir: Option<String>, // btc block dir, file name: <height>.hex
    pub btc_block_start_height: Option<u64>, // btc block start height
    pub btc_rpc_url: Option<String>,
    pub btc_rpc_username: Option<String>,
    pub btc_rpc_password: Option<String>,
    pub pprof_output: Option<PProfOutput>, // flamegraph(default)/proto
}

impl Default for BenchTxConfig {
    fn default() -> Self {
        Self {
            tx_type: Some(TxType::Empty),
            btc_block_dir: Some("target/btc_blocks".to_string()),
            btc_block_start_height: Some(820000),
            btc_rpc_url: None,
            btc_rpc_username: None,
            btc_rpc_password: None,
            pprof_output: Some(PProfOutput::Flamegraph),
        }
    }
}

impl BenchTxConfig {
    pub fn merge(&mut self, config: BenchTxConfig) {
        if config.tx_type.is_some() {
            self.tx_type = config.tx_type;
        }
        if config.btc_block_dir.is_some() {
            self.btc_block_dir = config.btc_block_dir;
        }
        if config.btc_block_start_height.is_some() {
            self.btc_block_start_height = config.btc_block_start_height;
        }
        if config.btc_rpc_url.is_some() {
            self.btc_rpc_url = config.btc_rpc_url;
        }
        if config.btc_rpc_username.is_some() {
            self.btc_rpc_username = config.btc_rpc_username;
        }
        if config.btc_rpc_password.is_some() {
            self.btc_rpc_password = config.btc_rpc_password;
        }
        if config.pprof_output.is_some() {
            self.pprof_output = config.pprof_output;
        }
    }

    pub fn load() -> Self {
        let path = &*BENCH_TX_CONFIG_PATH;
        let mut config = BenchTxConfig::default();
        match std::fs::read_to_string(path) {
            Ok(config_data) => match toml::from_str::<BenchTxConfig>(&config_data) {
                Ok(parsed_config) => config.merge(parsed_config),
                Err(e) => {
                    tracing::error!("Failed to parse config file: {}", e);
                }
            },
            Err(e) => {
                tracing::error!("Failed to read config file: {}", e);
            }
        };
        // Override config with env variables
        if let Ok(tx_type) = std::env::var("ROOCH_BENCH_TX_TYPE") {
            config.tx_type = Some(tx_type.parse().unwrap());
        }
        if let Ok(btc_block_dir) = std::env::var("ROOCH_BENCH_BTC_BLOCK_DIR") {
            config.btc_block_dir = Some(btc_block_dir);
        }
        if let Ok(btc_block_start_height) = std::env::var("ROOCH_BENCH_BTC_BLOCK_START_HEIGHT") {
            config.btc_block_start_height = Some(btc_block_start_height.parse().unwrap());
        }
        if let Ok(btc_rpc_url) = std::env::var("ROOCH_BENCH_BTC_RPC_URL") {
            config.btc_rpc_url = Some(btc_rpc_url);
        }
        if let Ok(btc_rpc_username) = std::env::var("ROOCH_BENCH_BTC_RPC_USERNAME") {
            config.btc_rpc_username = Some(btc_rpc_username);
        }
        if let Ok(btc_rpc_password) = std::env::var("ROOCH_BENCH_BTC_RPC_PASSWORD") {
            config.btc_rpc_password = Some(btc_rpc_password);
        }
        if let Ok(pprof_output) = std::env::var("ROOCH_BENCH_PPROF_OUTPUT") {
            config.pprof_output = Some(pprof_output.parse().unwrap());
        }
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
