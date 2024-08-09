// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::object::ObjectID;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepairIndexerType {
    Transaction,
    Event,
    ObjectState,
}

impl Display for RepairIndexerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairIndexerType::Transaction => write!(f, "transaction"),
            RepairIndexerType::Event => write!(f, "event"),
            RepairIndexerType::ObjectState => write!(f, "object_state"),
        }
    }
}

impl FromStr for RepairIndexerType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "transaction" => Ok(RepairIndexerType::Transaction),
            "event" => Ok(RepairIndexerType::Event),
            "object_state" => Ok(RepairIndexerType::ObjectState),
            s => Err(format_err!("Invalid repair indexer type str: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepairIndexerParams {
    /// Repair by owner.
    Owner(AccountAddress),
    /// Repair by object ids.
    ObjectId(Vec<ObjectID>),
}
