// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::BitcoinAddress;
use bitcoin::BlockHash;
use framework_builder::stdlib_version::StdlibVersion;
use move_core_types::value::MoveTypeLayout;
use moveos_types::{
    moveos_std::{module_store::ModuleStore, timestamp::Timestamp},
    state::{MoveState, ObjectState},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GenesisConfig {
    /// The Bitcoin network that the genesis block is based on
    pub bitcoin_network: u8,
    /// The height of the Bitcoin block that the genesis block is based on
    pub bitcoin_block_height: u64,
    /// The hash of the Bitcoin block that the genesis block is based on
    pub bitcoin_block_hash: BlockHash,
    /// The maximum number of blocks that can be reorganized
    pub bitcoin_reorg_block_count: u64,
    /// The timestamp of the Bitcoin block that the genesis block is based on
    pub timestamp: u64,
    pub sequencer_account: BitcoinAddress,
    pub genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    pub stdlib_version: StdlibVersion,
}

impl GenesisConfig {
    pub fn new(
        bitcoin_network: u8,
        bitcoin_block_height: u64,
        bitcoin_block_hash: BlockHash,
        bitcoin_reorg_block_count: u64,
        timestamp: u64,
        sequencer_account: BitcoinAddress,
        genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
        stdlib_version: StdlibVersion,
    ) -> Self {
        Self {
            bitcoin_network,
            bitcoin_block_height,
            bitcoin_block_hash,
            bitcoin_reorg_block_count,
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
    // The regtest genesis block hash
    bitcoin_block_hash: BlockHash::from_str(
        "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
    )
    .expect("Should be valid"),
    bitcoin_reorg_block_count: 0,
    timestamp: 0,
    sequencer_account: BitcoinAddress::default(),
    genesis_objects: vec![
        (
            ObjectState::new_timestamp(Timestamp { milliseconds: 0 }),
            Timestamp::type_layout(),
        ),
        (ObjectState::new_module_store(), ModuleStore::type_layout()),
    ],
    stdlib_version: StdlibVersion::Latest,
});

pub static G_DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    // The regtest genesis block hash
    bitcoin_block_hash: BlockHash::from_str(
        "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
    )
    .expect("Should be valid"),
    bitcoin_reorg_block_count: 0,
    timestamp: 0,
    sequencer_account: BitcoinAddress::from_str(
        "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
    )
    .expect("Should be valid"),
    genesis_objects: vec![
        (
            ObjectState::new_timestamp(Timestamp { milliseconds: 0 }),
            Timestamp::type_layout(),
        ),
        (ObjectState::new_module_store(), ModuleStore::type_layout()),
    ],
    stdlib_version: StdlibVersion::Latest,
});

pub static G_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    //curl -sSL https://mempool.space/testnet/api/blocks/tip
    //curl -sSL https://mempool.space/testnet/api/block/$(curl -sSL https://mempool.space/testnet/api/block-height/2867700)
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
        bitcoin_block_height: 2867700,
        bitcoin_block_hash: BlockHash::from_str(
            "0000000000000022704ba8fa408778fd482ed9a6d36e65847b26aea709fd4233",
        )
        .expect("Should be valid"),
        bitcoin_reorg_block_count: 3,
        //Make sure this timestamp is the same as Genesis Object Timestamp
        timestamp: 1720806147000,
        sequencer_account: BitcoinAddress::from_str(
            "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
        )
        .expect("Should be valid"),
        genesis_objects: vec![
            (
                ObjectState::new_timestamp(Timestamp {
                    milliseconds: 1720806147000,
                }),
                Timestamp::type_layout(),
            ),
            (ObjectState::new_module_store(), ModuleStore::type_layout()),
        ],
        stdlib_version: StdlibVersion::Version(1),
    }
});

pub static G_MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    //TODO define main config
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Bitcoin.to_num(),
        bitcoin_block_height: 0,
        bitcoin_block_hash: BlockHash::from_str(
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        )
        .expect("Should be valid"),
        bitcoin_reorg_block_count: 3,
        timestamp: 0,
        sequencer_account: BitcoinAddress::from_str(
            "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
        )
        .expect("Should be valid"),
        genesis_objects: vec![
            (
                ObjectState::new_timestamp(Timestamp { milliseconds: 0 }),
                Timestamp::type_layout(),
            ),
            (ObjectState::new_module_store(), ModuleStore::type_layout()),
        ],
        stdlib_version: StdlibVersion::Version(1),
    }
});
