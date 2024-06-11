// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::BitcoinAddress;
use ethers::types::H256;
use framework_builder::stdlib_version::StdlibVersion;
use move_core_types::language_storage::StructTag;
use moveos_types::moveos_std::object::ObjectID;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct GenesisObject {
    pub id: ObjectID,
    pub object_type: StructTag,
    pub state_root: H256,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct GenesisConfig {
    pub bitcoin_network: u8,
    pub bitcoin_block_height: u64,
    pub timestamp: u64,
    pub sequencer_account: BitcoinAddress,
    pub genesis_objects: Vec<GenesisObject>,
    pub stdlib_version: StdlibVersion,
}

impl GenesisConfig {
    pub fn new(
        bitcoin_network: u8,
        bitcoin_block_height: u64,
        timestamp: u64,
        sequencer_account: BitcoinAddress,
        genesis_objects: Vec<GenesisObject>,
        stdlib_version: StdlibVersion,
    ) -> Self {
        Self {
            bitcoin_network,
            bitcoin_block_height,
            timestamp,
            sequencer_account,
            genesis_objects,
            stdlib_version,
        }
    }

    pub fn load<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let content = std::fs::read_to_string(path)?;
        let config: GenesisConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

pub static G_LOCAL_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    timestamp: 0,
    sequencer_account: BitcoinAddress::default(),
    genesis_objects: vec![],
    stdlib_version: StdlibVersion::Latest,
});

pub static G_DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    timestamp: 0,
    sequencer_account: BitcoinAddress::from_str(
        "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
    )
    .expect("Should be valid"),
    genesis_objects: vec![],
    stdlib_version: StdlibVersion::Latest,
});

pub static G_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    // curl -sSL "https://mempool.space/testnet/api/block/000000008dfa22c53891f9a7b48a75ae8a523dec558873b061a321fd03a93bac"

    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
        bitcoin_block_height: 2819132,
        timestamp: 1717208620000,
        sequencer_account: BitcoinAddress::from_str(
            "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
        )
        .expect("Should be valid"),
        genesis_objects: vec![],
        stdlib_version: StdlibVersion::Version(1),
    }
});

pub static G_MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    //TODO define main config
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Bitcoin.to_num(),
        bitcoin_block_height: 0,
        timestamp: 0,
        sequencer_account: BitcoinAddress::from_str(
            "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
        )
        .expect("Should be valid"),
        genesis_objects: vec![],
        stdlib_version: StdlibVersion::Version(1),
    }
});
