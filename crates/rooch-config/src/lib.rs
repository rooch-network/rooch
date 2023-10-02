// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod server_config;
pub mod store_config;

use crate::store_config::StoreConfig;
use anyhow::Result;
use clap::Parser;
use moveos_config::{temp_dir, DataDirPath};
use once_cell::sync::Lazy;
use rooch_types::chain_id::RoochChainID;
use rooch_types::crypto::RoochKeyPair;
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::sync::Arc;
use std::{fmt::Debug, path::Path, path::PathBuf};

pub const ROOCH_DIR: &str = ".rooch";
pub const ROOCH_CONFIR_DIR: &str = "rooch_config";
pub const ROOCH_CLIENT_CONFIG: &str = "rooch.yaml";
pub const ROOCH_SERVER_CONFIG: &str = "server.yaml";
pub const ROOCH_KEYSTORE_FILENAME: &str = "rooch.keystore";

pub static R_DEFAULT_BASE_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs_next::home_dir()
        .expect("read home dir should ok")
        .join(".rooch")
});

pub fn rooch_config_dir() -> Result<PathBuf, anyhow::Error> {
    get_rooch_config_dir().and_then(|dir| {
        if !dir.exists() {
            create_dir_all(dir.clone())?;
        }
        Ok(dir)
    })
}

pub fn get_rooch_config_dir() -> Result<PathBuf, anyhow::Error> {
    match std::env::var_os("ROOCH_CONFIR_DIR") {
        Some(config_env) => Ok(config_env.into()),
        None => match dirs::home_dir() {
            Some(v) => Ok(v.join(ROOCH_DIR).join(ROOCH_CONFIR_DIR)),
            None => anyhow::bail!("Cannot obtain home directory path"),
        },
    }
}

pub static R_OPT_NET_HELP: &str = r#"Chain Network
    Builtin network: dev,test,main
    Custom network format: chain_name:chain_id
    Such as:
    my_chain:123 will init a new chain with id `123`.
    Custom network first start should also set the `genesis-config` option.
    Use rooch_generator command to generate a genesis config."#;

#[derive(Clone, Debug, Parser, Default, Serialize, Deserialize)]
pub struct RoochOpt {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "data-dir", short = 'd', parse(from_os_str))]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(flatten)]
    pub store: StoreConfig,

    /// Optional custom port, which the rooch server should listen on.
    /// The port on which the server should listen defaults to `50051`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'p')]
    pub port: Option<u16>,

    /// The Ethereum RPC URL to connect to for relay L1 block and transaction to L2.
    /// If not set, the relayer service will not start.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long)]
    pub eth_rpc_url: Option<String>,

    /// The address of the Rooch account to be use as key address
    #[clap(short = 'k', long = "key-address")]
    pub key_address: Option<String>,
}

impl std::fmt::Display for RoochOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
}

impl RoochOpt {
    pub fn new_with_temp_store() -> Self {
        RoochOpt {
            base_data_dir: Some(PathBuf::from("TMP")),
            chain_id: Some(RoochChainID::LOCAL),
            store: StoreConfig::default(),
            port: None,
            eth_rpc_url: None,
            key_address: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaseConfig {
    pub chain_id: RoochChainID,
    pub base_data_dir: DataDirPath,
    pub data_dir: PathBuf,
}

impl BaseConfig {
    pub fn load_with_opt(opt: &RoochOpt) -> Result<Self> {
        let chain_id = opt.chain_id.clone().unwrap_or_default();
        let base_data_dir = match opt.base_data_dir.clone() {
            Some(base_data_dir) if base_data_dir.to_str() == Some("TMP") => temp_dir(),
            Some(base_data_dir) => DataDirPath::PathBuf(base_data_dir),
            None => DataDirPath::PathBuf(R_DEFAULT_BASE_DATA_DIR.to_path_buf()),
        };

        let data_dir = base_data_dir.as_ref().join(chain_id.dir_name());
        if !data_dir.exists() {
            create_dir_all(data_dir.as_path())?
        }

        Ok(Self {
            chain_id,
            base_data_dir,
            data_dir,
        })
    }

    pub fn chain_id(&self) -> &RoochChainID {
        &self.chain_id
    }
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }
    pub fn base_data_dir(&self) -> DataDirPath {
        self.base_data_dir.clone()
    }
}

pub trait ConfigModule: Sized {
    /// Init the skip field or overwrite config by global command line option.
    fn merge_with_opt(&mut self, _opt: &RoochOpt, _base: Arc<BaseConfig>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Parser, Default, Serialize, Deserialize)]
pub struct ServerOpt {
    /// Sequencer, proposer and relayer keypair
    pub sequencer_keypair: Option<RoochKeyPair>,
    pub proposer_keypair: Option<RoochKeyPair>,
    pub relayer_keypair: Option<RoochKeyPair>,
}

impl std::fmt::Display for ServerOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
}

impl ServerOpt {
    pub fn new() -> Self {
        ServerOpt {
            sequencer_keypair: None,
            proposer_keypair: None,
            relayer_keypair: None,
        }
    }
}
