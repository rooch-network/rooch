// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::{ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{MoveStructState, MoveStructType};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("data_import_config");

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u8)]
pub enum DataImportMode {
    None = 0,
    UTXO = 1,
    Ord = 2,
}

impl TryFrom<u8> for DataImportMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DataImportMode::None),
            1 => Ok(DataImportMode::UTXO),
            2 => Ok(DataImportMode::Ord),
            _ => Err(anyhow::anyhow!(
                "Bitcoin data import mode {} is invalid",
                value
            )),
        }
    }
}

impl FromStr for DataImportMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utxo mode" => Ok(DataImportMode::UTXO),
            "ord mode" => Ok(DataImportMode::Ord),
            "none mode" => Ok(DataImportMode::None),
            _ => Err(anyhow::anyhow!("Invalid bitcoin data import mode {}", s)),
        }
    }
}

impl std::fmt::Display for DataImportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataImportMode::None => write!(f, "none mode"),
            DataImportMode::UTXO => write!(f, "utxo mode"),
            DataImportMode::Ord => write!(f, "ord mode"),
        }
    }
}

impl DataImportMode {
    pub fn to_num(self) -> u8 {
        self as u8
    }

    pub fn is_ord_mode(&self) -> bool {
        *self == DataImportMode::Ord
    }

    pub fn is_data_import_flag(&self) -> bool {
        *self == DataImportMode::UTXO || *self == DataImportMode::Ord
    }
}

impl Default for DataImportMode {
    // default bitcoin none modes
    fn default() -> Self {
        Self::None
    }
}

/// The Bitcoin data import mode onchain configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataImportConfig {
    pub data_import_mode: u8,
}

impl MoveStructType for DataImportConfig {
    const ADDRESS: move_core_types::account_address::AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DataImportConfig");
}

impl MoveStructState for DataImportConfig {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U8,
        ])
    }
}

impl DataImportConfig {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}
