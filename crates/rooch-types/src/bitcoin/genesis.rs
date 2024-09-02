// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{address::BitcoinAddress, addresses::BITCOIN_MOVE_ADDRESS};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("genesis");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultisignAccountConfig {
    pub multisign_bitcoin_address: BitcoinAddress,
    pub threshold: u64,
    pub participant_public_keys: Vec<Vec<u8>>,
}

impl MoveStructType for MultisignAccountConfig {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultisignAccountConfig");
}

impl MoveStructState for MultisignAccountConfig {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            BitcoinAddress::type_layout(),
            u64::type_layout(),
            Vec::<Vec<u8>>::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinGenesisContext {
    /// The bitcoin network
    pub network: u8,
    pub genesis_block_height: u64,
    pub genesis_block_hash: AccountAddress,
    pub reorg_block_count: u64,
    pub rooch_dao: MultisignAccountConfig,
}

impl MoveStructType for BitcoinGenesisContext {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinGenesisContext");
}

impl MoveStructState for BitcoinGenesisContext {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U8,
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Address,
            move_core_types::value::MoveTypeLayout::U64,
            MultisignAccountConfig::type_layout(),
        ])
    }
}

impl BitcoinGenesisContext {
    pub fn new(
        network: u8,
        genesis_block_height: u64,
        genesis_block_hash: AccountAddress,
        reorg_block_count: u64,
        rooch_dao: MultisignAccountConfig,
    ) -> Self {
        Self {
            network,
            genesis_block_height,
            genesis_block_hash,
            reorg_block_count,
            rooch_dao,
        }
    }
}
