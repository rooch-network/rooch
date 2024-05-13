// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::H256;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use moveos_types::moveos_std::object::ObjectID;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub const ALL_STDLIB_PACKAGE_NAMES: [&str; 5] = [
    "MoveStdlib",
    "MoveosStdlib",
    "RoochFramework",
    "BitcoinMove",
    "RoochNursery",
];

pub const ALL_STDLIB_PACKAGE_NAMES_STABLE: [&str; 4] = [
    "MoveStdlib",
    "MoveosStdlib",
    "RoochFramework",
    "BitcoinMove",
];

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
    pub sequencer_account: AccountAddress,
    pub genesis_objects: Vec<GenesisObject>,
    pub stdlib_package_names: Vec<String>,
}

pub static G_LOCAL_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    timestamp: 0,
    sequencer_account: AccountAddress::ONE,
    genesis_objects: vec![],
    stdlib_package_names: ALL_STDLIB_PACKAGE_NAMES
        .iter()
        .copied()
        .map(|s| s.to_owned())
        .collect(),
});

pub static G_DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
    bitcoin_block_height: 0,
    timestamp: 0,
    sequencer_account: AccountAddress::ONE,
    genesis_objects: vec![],
    stdlib_package_names: ALL_STDLIB_PACKAGE_NAMES
        .iter()
        .copied()
        .map(|s| s.to_owned())
        .collect(),
});

pub static G_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    //TODO define test config
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
        bitcoin_block_height: 0,
        timestamp: 0,
        sequencer_account: AccountAddress::ONE,
        genesis_objects: vec![],
        stdlib_package_names: ALL_STDLIB_PACKAGE_NAMES_STABLE
            .iter()
            .copied()
            .map(|s| s.to_owned())
            .collect(),
    }
});

pub static G_MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    //TODO define main config
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Bitcoin.to_num(),
        bitcoin_block_height: 0,
        timestamp: 0,
        sequencer_account: AccountAddress::ONE,
        genesis_objects: vec![],
        stdlib_package_names: ALL_STDLIB_PACKAGE_NAMES_STABLE
            .iter()
            .copied()
            .map(|s| s.to_owned())
            .collect(),
    }
});
