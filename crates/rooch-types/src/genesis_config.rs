// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{address::BitcoinAddress, bitcoin::genesis::MultisignAccountConfig};
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
    /// The genesis sequencer account
    pub sequencer_account: BitcoinAddress,
    /// The genesis rooch dao account multisign config
    pub rooch_dao: MultisignAccountConfig,
    pub genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    pub stdlib_version: StdlibVersion,
}

impl GenesisConfig {
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

//Note: on rooch, we do not distinguish the Bitcoin address format,
//So, we can use the mainnet address format for all networks

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
    // The local sequencer account and rooch dao account
    // will be update when the server first start
    sequencer_account: BitcoinAddress::from_str(
        "bc1pxup9p7um3t5knqn0yxfrq5d0mgul9ts993j32tsfxn68qa4pl3nq2qhh2e",
    )
    .unwrap(),
    rooch_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1pevdrc8yqmgd94h2mpz9st0u77htmx935hzck3ruwsvcf4w7wrnqqd0yvze",
        )
        .unwrap(),
        threshold: 1,
        participant_public_keys: vec![hex::decode(
            "03ff7e1d7b4a152671124545f4fb68efe2a9bd0b3870ac22fee4afd4ecdfa8a19c",
        )
        .unwrap()],
    },
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
        "bc1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasq8lj2sg",
    )
    .expect("Should be valid"),
    rooch_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1pu38mumfnuppqn54kcnyymmqzpqgmmfxlgnu6dsc6qhschy7cj76qkcl24p",
        )
        .unwrap(),
        threshold: 1,
        // Dev multisign account public key is the same as sequencer account
        participant_public_keys: vec![hex::decode(
            "026c9e5a00643a706d3826424f766bbbb08adada4dc357c1b279ad4662d2fd1e2e",
        )
        .unwrap()],
    },
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
            "bc1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasq8lj2sg",
        )
        .expect("Should be valid"),
        rooch_dao: MultisignAccountConfig {
            multisign_bitcoin_address: BitcoinAddress::from_str(
                "bc1pmk3767wmd3pjhnyeajckqpdgk8uxp8mmc2vqxa54at77ykml888sllqrpk",
            )
            .unwrap(),
            threshold: 2,
            participant_public_keys: vec![
                hex::decode("026c9e5a00643a706d3826424f766bbbb08adada4dc357c1b279ad4662d2fd1e2e")
                    .unwrap(),
                hex::decode("030fd15c2abac203b2e819cc7cd95b0206e200c61b360027160cdb3c727d071968")
                    .unwrap(),
                hex::decode("039ef579cd92104255ea7e18abbc1c761f9cd40f19d940d72e6849c6b2b930e7f0")
                    .unwrap(),
            ],
        },
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
        stdlib_version: StdlibVersion::Version(8),
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
    //TODO update sequencer account
    sequencer_account: BitcoinAddress::from_str(
        "bc1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasq8lj2sg",
    )
    .expect("Should be valid"),
    //TODO update rooch dao multisign account
    rooch_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1pmk3767wmd3pjhnyeajckqpdgk8uxp8mmc2vqxa54at77ykml888sllqrpk",
        )
        .unwrap(),
        threshold: 2,
        participant_public_keys: vec![
            hex::decode("026c9e5a00643a706d3826424f766bbbb08adada4dc357c1b279ad4662d2fd1e2e")
                .unwrap(),
            hex::decode("030fd15c2abac203b2e819cc7cd95b0206e200c61b360027160cdb3c727d071968")
                .unwrap(),
            hex::decode("039ef579cd92104255ea7e18abbc1c761f9cd40f19d940d72e6849c6b2b930e7f0")
                .unwrap(),
        ],
    },
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
    stdlib_version: StdlibVersion::Version(8),
});

#[cfg(test)]
mod tests {
    use super::GenesisConfig;
    use crate::{bitcoin::multisign_account, crypto::RoochKeyPair};
    use moveos_types::h256::sha2_256_of;

    fn test_genesis_config(config: &GenesisConfig) {
        //println!("sequence_account: {}", config.sequencer_account);
        let multisign_account = multisign_account::generate_multisign_address(
            config.rooch_dao.threshold as usize,
            config.rooch_dao.participant_public_keys.clone(),
        )
        .unwrap();
        //println!("multisign_account: {}", multisign_account);
        assert_eq!(
            config.rooch_dao.multisign_bitcoin_address,
            multisign_account
        );
    }

    #[test]
    fn test_genesis_config_dev() {
        test_genesis_config(&super::G_DEV_CONFIG);
    }

    #[test]
    fn test_genesis_config_test() {
        test_genesis_config(&super::G_TEST_CONFIG);
    }

    #[test]
    fn test_genesis_config_main() {
        test_genesis_config(&super::G_MAIN_CONFIG);
    }

    // We use the hash of "RoochNetwork" as the private key seed
    // To generate the public key, bitcoin address and rooch address for unit test usage
    #[test]
    fn test_unit_test_genesis_config() {
        let hash = sha2_256_of("RoochNetwork".as_bytes());
        let private_key = RoochKeyPair::from_secp256k1_bytes(hash.as_bytes()).unwrap();
        let public_key = private_key.bitcoin_public_key().unwrap();
        let bitcoin_address = private_key.public().bitcoin_address().unwrap();
        let rooch_address = bitcoin_address.to_rooch_address();

        //println!("public key:{}", public_key);
        assert_eq!(
            public_key.to_bytes(),
            hex::decode("03ff7e1d7b4a152671124545f4fb68efe2a9bd0b3870ac22fee4afd4ecdfa8a19c")
                .unwrap()
        );
        //println!("bitcoin_address: {}", bitcoin_address);
        assert_eq!(
            bitcoin_address.to_string(),
            "bc1pxup9p7um3t5knqn0yxfrq5d0mgul9ts993j32tsfxn68qa4pl3nq2qhh2e".to_owned()
        );
        //println!("rooch_address: {:?}", rooch_address);
        assert_eq!(
            rooch_address.to_hex_literal(),
            "0x76ce23dc5aee99b1d880193abd85c8d9511856677532c8b5c224b0e748b2010f"
        );
        let multisign_bitcoin_address =
            multisign_account::generate_multisign_address(1, vec![public_key.to_bytes().to_vec()])
                .unwrap();
        //println!("multisign_account: {}", multisign_account);
        assert_eq!(
            multisign_bitcoin_address.to_string(),
            "bc1pevdrc8yqmgd94h2mpz9st0u77htmx935hzck3ruwsvcf4w7wrnqqd0yvze"
        );
        let multisign_address = multisign_bitcoin_address.to_rooch_address();
        //println!("multisign_address: {:?}", multisign_address);
        assert_eq!(
            multisign_address.to_hex_literal(),
            "0xd10165036fe6e26dc68b9aad7b6c979bb952d8b0ed9c583bb7e6ff13ee321f69"
        );
    }
}
