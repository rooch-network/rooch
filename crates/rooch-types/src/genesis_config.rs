// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::H256;
use framework_builder::stdlib_version::StdlibVersion;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use moveos_types::moveos_std::object::ObjectID;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

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
    pub stdlib_version: StdlibVersion,
}

pub static G_LOCAL_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    timestamp: 0,
    sequencer_account: AccountAddress::ONE,
    genesis_objects: vec![],
    stdlib_version: StdlibVersion::Latest,
});

pub static G_DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
    bitcoin_block_height: 0,
    timestamp: 0,
    sequencer_account: AccountAddress::ONE,
    genesis_objects: vec![],
    stdlib_version: StdlibVersion::Latest,
});

pub static G_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    // curl -sSL "https://mempool.space/testnet/api/block/000000009373df1134670bd45e75a6b8f1fb07610a9eeb7933ef266da9507cb9"
    GenesisConfig {
        bitcoin_network: crate::bitcoin::network::Network::Testnet.to_num(),
        bitcoin_block_height: 2815983,
        timestamp: 1715941066000,
        sequencer_account: AccountAddress::from_hex_literal(
            "0xbe2701d15ccdc282caf8ca6647e7a54db5721f8bcb7b980b4d0c65a151bf74da",
        )
        .expect("Invalid address"),
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
        sequencer_account: AccountAddress::ONE,
        genesis_objects: vec![],
        stdlib_version: StdlibVersion::Version(1),
    }
});
