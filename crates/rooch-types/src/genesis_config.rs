// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::BitcoinAddress,
    bitcoin::{genesis::MultisignAccountConfig, ord::InscriptionStore, utxo::BitcoinUTXOStore},
    framework::address_mapping::RoochToBitcoinAddressMapping,
};
use bitcoin::{block::Header, BlockHash};
use framework_builder::stdlib_version::StdlibVersion;
use move_core_types::value::MoveTypeLayout;
use moveos_types::{
    h256::H256,
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
        // "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f", // mainnet 0 block hash
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

// curl -sSL "https://mempool.space/testnet/api/block/$(curl -sSL https://mempool.space/testnet/api/block-height/3490876)/header"
static TESTNET_GENESIS_HEIGHT_HEADER: Lazy<(u64, Header)> = Lazy::new(|| {
    (3490876, bitcoin::consensus::deserialize(
        &hex::decode("00000020ad001d713c5fa9930589d22ab68e830303201ec842b3ef390100000000000000d997f8d470a5bf41a94cc6ffb835c217d11af1b8ebe31f30a96923ee84d16e64bbca4967ffff001d0e81faa0")
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
                "bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt",
            )
            .unwrap(),
            threshold: 5,
            participant_public_keys: vec![
                hex::decode("032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c")
                    .unwrap(),
                hex::decode("039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0")
                    .unwrap(),
                hex::decode("03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61")
                    .unwrap(),
                hex::decode("0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677")
                    .unwrap(),
                hex::decode("03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be")
                    .unwrap(),
                hex::decode("0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db")
                    .unwrap(),
                hex::decode("028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a")
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

// curl -sSL "https://mempool.space/api/block/$(curl -sSL https://mempool.space/api/block-height/859001)/header"
static MAIN_GENESIS_HEIGHT_HEADER: Lazy<(u64, Header)> = Lazy::new(|| {
    (859001, bitcoin::consensus::deserialize(
        &hex::decode("00e0ff274e6e46285bf4133faaafcf248ed461ffcdf8e2b33fba020000000000000000004275ffbb1e17c5b8abb04a9e57bc479c83dcf44c7bed3bc7f94c8449b6c2250619ecd0665b250317b7bc8d78")
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
        "bc1pwxpq9pxgv2jnvzu2pjska3jkfurxsdt075yds3u0rsj9cu39g4esjdzt8z",
    )
    .expect("Should be valid"),
    rooch_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt",
        )
        .unwrap(),
        threshold: 5,
        participant_public_keys: vec![
            hex::decode("032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c")
                .unwrap(),
            hex::decode("039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0")
                .unwrap(),
            hex::decode("03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61")
                .unwrap(),
            hex::decode("0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677")
                .unwrap(),
            hex::decode("03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be")
                .unwrap(),
            hex::decode("0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db")
                .unwrap(),
            hex::decode("028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a")
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
        (
            BitcoinUTXOStore::genesis_with_state_root(
                H256::from_str(
                    "0x8ec77de7cd44c27a30c84aaa36c4e107aae7aaade2ae3ee1741aad437015a219",
                )
                .unwrap(),
                185390577,
            ),
            BitcoinUTXOStore::type_layout(),
        ),
        (
            InscriptionStore::genesis_with_state_root(
                H256::from_str(
                    "0x8a4fc2cfb4d66c574e921b4fffa1a8af9156f821451cac1f3d61075572cdf68b",
                )
                .unwrap(),
                150953628,
                InscriptionStore {
                    cursed_inscription_count: 472043,
                    blessed_inscription_count: 75004771,
                    unbound_inscription_count: 20723,
                    lost_sats: 0,
                    next_sequence_number: 75476814,
                },
            ),
            InscriptionStore::type_layout(),
        ),
        (
            RoochToBitcoinAddressMapping::genesis_with_state_root(
                H256::from_str(
                    "0x908b63a475a886571a2bef1533589866f92fb3ef01b243a0b8bb1cda27655172",
                )
                .unwrap(),
                52397723,
            ),
            RoochToBitcoinAddressMapping::type_layout(),
        ),
    ],
    stdlib_version: StdlibVersion::Version(11),
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
        println!("multisign_account: {}", multisign_account);
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
