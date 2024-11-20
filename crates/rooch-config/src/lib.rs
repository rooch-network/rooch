// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::da_config::DAConfig;
use crate::proposer_config::ProposerConfig;
use crate::store_config::StoreConfig;
use anyhow::Result;
use clap::Parser;
use moveos_config::{temp_dir, DataDirPath};
use once_cell::sync::Lazy;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::genesis_config::GenesisConfig;
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID, RoochNetwork};
use rooch_types::service_status::ServiceStatus;
use rooch_types::service_type::ServiceType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt::Debug, path::Path, path::PathBuf};

pub mod config;
pub mod da_config;
pub mod proposer_config;
pub mod server_config;
pub mod settings;
pub mod store_config;

pub const ROOCH_DIR: &str = ".rooch";
pub const ROOCH_CONFIR_DIR: &str = "rooch_config";
pub const ROOCH_CLIENT_CONFIG: &str = "rooch.yaml";
pub const ROOCH_KEYSTORE_FILENAME: &str = "rooch.keystore";

pub static R_DEFAULT_BASE_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs_next::home_dir()
        .expect("read home dir should ok")
        .join(".rooch")
});

#[derive(Debug, Clone, PartialEq)]
pub enum MapConfigValueSource {
    MapConfig,   // Value came from the presence of a key in the map configuration
    Environment, // Value came from the environment
    Default,     // Value came from a defined default value
    None,        // Value is not present in the map configuration, environment, or default value
}

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
    Builtin network: local,dev,test,main
    Custom network format: chain_id
    Such as:
    The Custom network need to use `rooch genesis init` command to init the genesis config first."#;

#[derive(Clone, Debug, Parser, Default, Serialize, Deserialize)]
pub struct RoochOpt {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    /// If not set, the default data dir is $HOME/.rooch
    /// If set to `TMP`, the service will start with a temporary data store.
    pub base_data_dir: Option<PathBuf>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long)]
    /// The genesis config file path for custom chain network.
    /// If the file path equals to builtin chain network name(local/dev/test/main), will use builtin genesis config.
    pub genesis_config: Option<String>,

    #[clap(flatten)]
    pub store: StoreConfig,

    /// Optional custom port, which the rooch server should listen on.
    /// The port on which the server should listen defaults to `6767`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'p')]
    pub port: Option<u16>,

    /// The Ethereum RPC URL to connect to for relay L1 block and transaction to L2.
    /// If not set, the relayer service will not start.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, env = "ETH_RPC_URL")]
    pub eth_rpc_url: Option<String>,

    /// The Bitcoin RPC URL to connect to for relay L1 block and transaction to L2.
    /// If not set, the relayer service will not start.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        long,
        env = "BITCOIN_RPC_URL",
        requires = "btc-rpc-username",
        requires = "btc-rpc-password"
    )]
    pub btc_rpc_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, id = "btc-rpc-username", env = "BTC_RPC_USERNAME")]
    pub btc_rpc_username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, id = "btc-rpc-password", env = "BTC_RPC_PASSWORD")]
    pub btc_rpc_password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, env = "BTC_END_BLOCK_HEIGHT")]
    /// The end block height of the Bitcoin chain to stop relaying from, default is none.
    pub btc_end_block_height: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, env = "BTC_SYNC_BLOCK_INTERVAL")]
    /// The interval of sync BTC block, default is none.
    pub btc_sync_block_interval: Option<u64>,

    /// The address of the sequencer account
    #[clap(long)]
    pub sequencer_account: Option<String>,
    /// The address of the proposer account
    #[clap(long)]
    pub proposer_account: Option<String>,

    #[clap(long, default_value_t)]
    pub da: DAConfig,

    #[clap(flatten)]
    pub proposer: ProposerConfig,

    #[clap(long, default_value_t, value_enum)]
    pub service_status: ServiceStatus,

    /// Set quota size that defines how many requests can occur
    /// before the governor middleware starts blocking requests from an IP address and
    /// clients have to wait until the elements of the quota are replenished.
    ///
    /// **The burst_size must not be zero.**
    #[clap(long)]
    pub traffic_burst_size: Option<u32>,

    /// Set the interval after which one element of the quota is replenished in seconds.
    ///
    /// **The interval must not be zero.**
    #[clap(long)]
    pub traffic_per_second: Option<u64>,

    #[clap(long, default_value_t, value_enum)]
    pub service_type: ServiceType,

    #[serde(skip)]
    #[clap(skip)]
    base: Option<Arc<BaseConfig>>,
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
    pub fn new_with_temp_store() -> Result<Self> {
        let mut opt = RoochOpt {
            base_data_dir: Some("TMP".into()),
            chain_id: Some(BuiltinChainID::Local.into()),
            genesis_config: None,
            store: StoreConfig::default(),
            port: None,
            eth_rpc_url: None,
            btc_rpc_url: None,
            btc_rpc_username: None,
            btc_rpc_password: None,
            btc_end_block_height: None,
            btc_sync_block_interval: None,
            sequencer_account: None,
            proposer_account: None,
            da: DAConfig::default(),
            proposer: ProposerConfig::default(),
            service_status: ServiceStatus::default(),
            traffic_per_second: None,
            traffic_burst_size: None,
            base: None,
            service_type: ServiceType::default(),
        };
        opt.init()?;
        Ok(opt)
    }

    pub fn new_with_default(
        base_data_dir: Option<PathBuf>,
        chain_id: Option<RoochChainID>,
        genesis_config: Option<String>,
    ) -> Result<Self> {
        let mut opt = RoochOpt {
            base_data_dir,
            chain_id,
            genesis_config,
            ..Default::default()
        };
        opt.init()?;
        Ok(opt)
    }

    pub fn init(&mut self) -> Result<()> {
        if self.base.is_none() {
            let base = BaseConfig::load_with_opt(self)?;
            let arc_base = Arc::new(base);
            self.store.init(Arc::clone(&arc_base))?;
            self.da.init(Arc::clone(&arc_base))?;
            self.base = Some(arc_base);
        }
        Ok(())
    }

    pub fn ethereum_relayer_config(&self) -> Option<EthereumRelayerConfig> {
        self.eth_rpc_url
            .as_ref()
            .map(|eth_rpc_url| EthereumRelayerConfig {
                eth_rpc_url: eth_rpc_url.clone(),
            })
    }

    pub fn bitcoin_relayer_config(&self) -> Option<BitcoinRelayerConfig> {
        self.btc_rpc_url.as_ref()?;
        Some(BitcoinRelayerConfig {
            btc_rpc_url: self.btc_rpc_url.clone().unwrap(),
            btc_rpc_user_name: self.btc_rpc_username.clone().unwrap(),
            btc_rpc_password: self.btc_rpc_password.clone().unwrap(),
            btc_end_block_height: self.btc_end_block_height,
            btc_sync_block_interval: self.btc_sync_block_interval,
        })
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(6767)
    }

    pub fn chain_id(&self) -> RoochChainID {
        self.chain_id.clone().unwrap_or_default()
    }

    pub fn genesis_config(&self) -> Option<GenesisConfig> {
        self.genesis_config.clone().map(|path| {
            let path = path.trim();
            let genesis_config: GenesisConfig = match BuiltinChainID::from_str(path) {
                Ok(builtin_id) => builtin_id.genesis_config().clone(),
                Err(_) => {
                    let content =
                        std::fs::read_to_string(path).expect("read genesis config file should ok");
                    serde_yaml::from_str(&content).expect("parse genesis config should ok")
                }
            };
            genesis_config
        })
    }

    pub fn network(&self) -> RoochNetwork {
        match self.chain_id() {
            RoochChainID::Builtin(id) => RoochNetwork::builtin(id),
            RoochChainID::Custom(id) => {
                let genesis_config = self
                    .genesis_config()
                    .expect("Genesis config is required for custom network.");
                RoochNetwork::new(id, genesis_config)
            }
        }
    }

    pub fn store_config(&self) -> &StoreConfig {
        &self.store
    }

    pub fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn da_config(&self) -> &DAConfig {
        &self.da
    }
}

#[derive(Debug, Clone)]
pub struct EthereumRelayerConfig {
    pub eth_rpc_url: String,
}

#[derive(Debug, Clone)]
pub struct BitcoinRelayerConfig {
    pub btc_rpc_url: String,
    pub btc_rpc_user_name: String,
    pub btc_rpc_password: String,
    pub btc_end_block_height: Option<u64>,
    pub btc_sync_block_interval: Option<u64>,
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
    pub fn base_data_dir(&self) -> &Path {
        self.base_data_dir.path()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServerOpt {
    /// Sequencer, proposer and relayer keypair
    pub sequencer_keypair: Option<RoochKeyPair>,
    pub proposer_keypair: Option<RoochKeyPair>,
    pub active_env: Option<String>,
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
            active_env: None,
        }
    }

    pub fn get_active_env(&self) -> String {
        self.active_env
            .clone()
            .unwrap_or_else(|| RoochChainID::default().chain_name())
    }
}

// value order:
// 1. config map
// 2. env value
// 3. default value
pub fn retrieve_map_config_value(
    map_config: &mut HashMap<String, String>,
    key: &str,
    env_var: Option<&str>,
    default_var: Option<&str>,
) -> MapConfigValueSource {
    if map_config.contains_key(key) {
        return MapConfigValueSource::MapConfig;
    }

    if let Some(env_var) = env_var {
        if let Ok(env_var_value) = std::env::var(env_var) {
            // env_var exists
            map_config.insert(key.to_string(), env_var_value.clone());
            return MapConfigValueSource::Environment;
        }
    }

    // Use the default
    if let Some(default_var) = default_var {
        map_config.insert(key.to_string(), default_var.to_string());
        return MapConfigValueSource::Default;
    }
    MapConfigValueSource::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    mod retrieve_map_config_value_tests {
        use super::*;
        use std::time;

        #[test]
        fn returns_map_config_when_key_exists() {
            let mut map_config = HashMap::new();
            map_config.insert("key1".to_string(), "value1".to_string());

            assert_eq!(
                retrieve_map_config_value(&mut map_config, "key1", None, Some("default")),
                MapConfigValueSource::MapConfig
            );
        }

        #[test]
        fn returns_default_when_key_does_not_exist_and_no_env_var() {
            let mut map_config = HashMap::new();

            assert_eq!(
                retrieve_map_config_value(&mut map_config, "key2", None, Some("default")),
                MapConfigValueSource::Default
            );
            assert_eq!(map_config.get("key2").unwrap(), "default");
        }

        #[test]
        fn returns_environment_when_env_var_exists() {
            let mut map_config = HashMap::new();

            // make a random env key
            let env_key = format!(
                "TEST_ENV_VAR_{}",
                time::SystemTime::now().elapsed().unwrap().as_secs()
            );

            env::set_var(env_key.clone(), "env_value");

            assert_eq!(
                retrieve_map_config_value(
                    &mut map_config,
                    "key2",
                    Some(&env_key.clone()),
                    Some("default")
                ),
                MapConfigValueSource::Environment
            );
            assert_eq!(map_config.get("key2").unwrap(), "env_value");

            env::remove_var(env_key);
        }

        #[test]
        fn returns_none_when_neither_key_nor_env_var_nor_default_exists() {
            let mut map_config = HashMap::new();
            assert_eq!(
                retrieve_map_config_value(&mut map_config, "key3", None, None),
                MapConfigValueSource::None
            );
        }
    }
}
