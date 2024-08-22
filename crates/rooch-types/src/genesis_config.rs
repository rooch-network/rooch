// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::BitcoinAddress;
use bitcoin::{block::Header, BlockHash};
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
        (
            ObjectState::genesis_module_store(),
            ModuleStore::type_layout(),
        ),
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
        (
            ObjectState::genesis_module_store(),
            ModuleStore::type_layout(),
        ),
    ],
    stdlib_version: StdlibVersion::Latest,
});

//curl -sSL https://mempool.space/testnet/api/block/$(curl -sSL https://mempool.space/testnet/api/block-height/2867700)/header
static TESTNET_GENESIS_HEIGHT_HEADER: Lazy<(u64, Header)> = Lazy::new(|| {
    (2867900, bitcoin::consensus::deserialize(
        &hex::decode("00e0962bd97a2b80ffb30abf34c2dc211c167a3e35dc6e5bdba5ac1d23208d6f0000000011059bafb1e9ceb8f2e494671c078863589574f5964548f4c0aa3ba0da733ebfa47f9266129422199e86b520")
            .expect("Should be valid"),
    ).expect("Should be valid"))
});

pub static G_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
        bitcoin_block_height: TESTNET_GENESIS_HEIGHT_HEADER.0,
        bitcoin_block_hash: TESTNET_GENESIS_HEIGHT_HEADER.1.block_hash(),
        bitcoin_reorg_block_count: 3,
        //Make sure this timestamp is the same as Genesis Object Timestamp
        timestamp: TESTNET_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
        sequencer_account: BitcoinAddress::from_str(
            "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
        )
        .expect("Should be valid"),
        genesis_objects: vec![
            (
                ObjectState::new_timestamp(Timestamp {
                    milliseconds: TESTNET_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
                }),
                Timestamp::type_layout(),
            ),
            (
                ObjectState::genesis_module_store(),
                ModuleStore::type_layout(),
            ),
        ],
        stdlib_version: StdlibVersion::Version(1),
    }
});

//curl -sSL https://mempool.space/api/block/$(curl -sSL https://mempool.space/api/block-height/852203)/header

static MAIN_GENESIS_HEIGHT_HEADER: Lazy<(u64, Header)> = Lazy::new(|| {
    (852203, bitcoin::consensus::deserialize(
        &hex::decode("0000002aeab3d78df1e6bdd612df31107b32470b2946758410920100000000000000000012c85f974f244dce28142748dd6dc0a8d2a8cdca6706442e6df4b3b52c0d985d794c94666d8a031729521ec6")
            .expect("Should be valid"),
    ).expect("Should be valid"))
});

pub static G_MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Bitcoin.to_num(),
    bitcoin_block_height: MAIN_GENESIS_HEIGHT_HEADER.0,
    bitcoin_block_hash: MAIN_GENESIS_HEIGHT_HEADER.1.block_hash(),
    bitcoin_reorg_block_count: 3,
    timestamp: MAIN_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
    sequencer_account: BitcoinAddress::from_str(
        "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
    )
    .expect("Should be valid"),
    genesis_objects: vec![
        (
            ObjectState::new_timestamp(Timestamp {
                milliseconds: MAIN_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
            }),
            Timestamp::type_layout(),
        ),
        (
            ObjectState::genesis_module_store(),
            ModuleStore::type_layout(),
        ),
    ],
    stdlib_version: StdlibVersion::Version(6),
});
