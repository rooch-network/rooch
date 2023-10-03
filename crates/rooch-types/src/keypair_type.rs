// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::ArgEnum;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::error::RoochError;

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Serialize,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    JsonSchema,
    EnumString,
    ArgEnum,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[strum(serialize_all = "lowercase")]
pub enum KeyPairType {
    RoochKeyPairType,
    EthereumKeyPairType,
}

impl KeyPairType {
    const ROOCH_KEY_PAIR_TYPE: &str = "rooch";
    const ETHEREUM_KEY_PAIR_TYPE: &str = "evm";

    pub fn type_of(&self) -> String {
        match self {
            KeyPairType::RoochKeyPairType => Self::ROOCH_KEY_PAIR_TYPE.to_owned(),
            KeyPairType::EthereumKeyPairType => Self::ETHEREUM_KEY_PAIR_TYPE.to_owned(),
        }
    }

    pub fn from_type(type_as: &str) -> Result<KeyPairType, RoochError> {
        let byte_int = type_as
            .parse::<String>()
            .map_err(|_| RoochError::KeyConversionError("Invalid type".to_owned()))?;
        Self::from_type_byte(byte_int.as_str())
    }

    pub fn from_type_byte(type_byte: &str) -> Result<KeyPairType, RoochError> {
        match type_byte {
            Self::ROOCH_KEY_PAIR_TYPE => Ok(KeyPairType::RoochKeyPairType),
            Self::ETHEREUM_KEY_PAIR_TYPE => Ok(KeyPairType::EthereumKeyPairType),
            _ => Err(RoochError::KeyConversionError("Invalid type".to_owned())),
        }
    }
}
