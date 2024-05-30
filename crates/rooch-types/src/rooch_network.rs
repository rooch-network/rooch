// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::framework::chain_id::ChainID;
use crate::genesis_config::{self, GenesisConfig};
use anyhow::{bail, format_err, Result};
use move_core_types::account_address::AccountAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const CHAIN_ID_LOCAL: u64 = 4;
pub const CHAIN_ID_DEV: u64 = 3;
pub const CHAIN_ID_TEST: u64 = 2;
pub const CHAIN_ID_MAIN: u64 = 1;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[repr(u64)]
pub enum BuiltinChainID {
    /// A temp network just for developer test.
    /// The data is stored in the temporary directory and will be cleared after restarting.
    #[default]
    Local = CHAIN_ID_LOCAL,
    /// A ephemeral network just for developer test.
    Dev = CHAIN_ID_DEV,
    /// Rooch test network.
    /// The data on the chain will be cleaned up periodically.
    Test = CHAIN_ID_TEST,
    /// Rooch main net.
    Main = CHAIN_ID_MAIN,
}

impl Display for BuiltinChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinChainID::Local => write!(f, "local"),
            BuiltinChainID::Dev => write!(f, "dev"),
            BuiltinChainID::Test => write!(f, "test"),
            BuiltinChainID::Main => write!(f, "main"),
        }
    }
}

impl From<BuiltinChainID> for u64 {
    fn from(chain_id: BuiltinChainID) -> Self {
        chain_id as u64
    }
}

impl TryFrom<u64> for BuiltinChainID {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            CHAIN_ID_LOCAL => Ok(BuiltinChainID::Local),
            CHAIN_ID_DEV => Ok(BuiltinChainID::Dev),
            CHAIN_ID_TEST => Ok(BuiltinChainID::Test),
            CHAIN_ID_MAIN => Ok(BuiltinChainID::Main),
            _ => Err(anyhow::anyhow!("chain id {} is invalid", value)),
        }
    }
}

impl FromStr for BuiltinChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(BuiltinChainID::Local),
            "dev" => Ok(BuiltinChainID::Dev),
            "test" => Ok(BuiltinChainID::Test),
            "main" => Ok(BuiltinChainID::Main),
            s => Err(format_err!("Unknown chain: {}", s)),
        }
    }
}

impl TryFrom<ChainID> for BuiltinChainID {
    type Error = anyhow::Error;
    fn try_from(id: ChainID) -> Result<Self, Self::Error> {
        Ok(match id.id() {
            CHAIN_ID_LOCAL => Self::Local,
            CHAIN_ID_DEV => Self::Dev,
            CHAIN_ID_TEST => Self::Test,
            CHAIN_ID_MAIN => Self::Main,
            id => bail!("{} is not a builtin chain id", id),
        })
    }
}

impl From<BuiltinChainID> for ChainID {
    fn from(chain_id: BuiltinChainID) -> Self {
        ChainID::new(chain_id.into())
    }
}

impl BuiltinChainID {
    pub fn chain_name(self) -> String {
        self.to_string().to_lowercase()
    }

    pub fn chain_id(self) -> ChainID {
        ChainID::new(self.into())
    }

    pub fn is_local(self) -> bool {
        matches!(self, BuiltinChainID::Local)
    }

    pub fn is_dev(self) -> bool {
        matches!(self, BuiltinChainID::Dev)
    }

    pub fn is_test(self) -> bool {
        matches!(self, BuiltinChainID::Test)
    }

    pub fn assert_test_or_dev_or_local(self) -> Result<()> {
        if !self.is_test_or_dev_or_local() {
            bail!("Only support test or dev or local network.")
        }
        Ok(())
    }

    pub fn is_test_or_dev_or_local(self) -> bool {
        matches!(
            self,
            BuiltinChainID::Test | BuiltinChainID::Dev | BuiltinChainID::Local
        )
    }

    pub fn is_main(self) -> bool {
        matches!(self, BuiltinChainID::Main)
    }

    pub fn chain_ids() -> Vec<BuiltinChainID> {
        vec![
            BuiltinChainID::Local,
            BuiltinChainID::Dev,
            BuiltinChainID::Test,
            BuiltinChainID::Main,
        ]
    }

    pub fn genesis_config(&self) -> &GenesisConfig {
        match self {
            BuiltinChainID::Local => &genesis_config::G_LOCAL_CONFIG,
            BuiltinChainID::Dev => &genesis_config::G_DEV_CONFIG,
            BuiltinChainID::Test => &genesis_config::G_TEST_CONFIG,
            BuiltinChainID::Main => &genesis_config::G_MAIN_CONFIG,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum RoochChainID {
    Builtin(BuiltinChainID),
    Custom(ChainID),
}

impl Display for RoochChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Builtin(b) => b.to_string(),
            Self::Custom(c) => c.to_string(),
        };
        write!(f, "{}", name)
    }
}

impl From<BuiltinChainID> for RoochChainID {
    fn from(chain_id: BuiltinChainID) -> Self {
        RoochChainID::Builtin(chain_id)
    }
}

impl From<ChainID> for RoochChainID {
    fn from(chain_id: ChainID) -> Self {
        match chain_id.id() {
            CHAIN_ID_LOCAL => RoochChainID::Builtin(BuiltinChainID::Local),
            CHAIN_ID_DEV => RoochChainID::Builtin(BuiltinChainID::Dev),
            CHAIN_ID_TEST => RoochChainID::Builtin(BuiltinChainID::Test),
            CHAIN_ID_MAIN => RoochChainID::Builtin(BuiltinChainID::Main),
            _ => RoochChainID::Custom(chain_id),
        }
    }
}

impl FromStr for RoochChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match BuiltinChainID::from_str(s) {
            Ok(chain_id) => Ok(Self::Builtin(chain_id)),
            Err(_e) => Ok(Self::Custom(ChainID::from_str(s)?)),
        }
    }
}

impl RoochChainID {
    pub fn chain_name(&self) -> String {
        match self {
            Self::Builtin(b) => b.chain_name(),
            Self::Custom(c) => c.to_string(),
        }
    }

    pub fn chain_id(&self) -> ChainID {
        match self {
            Self::Builtin(b) => b.chain_id(),
            Self::Custom(c) => c.clone(),
        }
    }

    pub fn assert_test_or_dev_or_local(&self) -> Result<()> {
        if !self.is_test_or_dev_or_local() {
            bail!("Only support test or dev or local chain_id.")
        }
        Ok(())
    }

    pub fn is_builtin(&self) -> bool {
        self.is_test() || self.is_dev() || self.is_main()
    }

    pub fn is_test_or_dev_or_local(&self) -> bool {
        self.is_test() || self.is_dev() || self.is_local()
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinChainID::Local))
    }

    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinChainID::Dev))
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinChainID::Test))
    }

    pub fn is_main(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinChainID::Main))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Default data dir name of this chain_id
    pub fn dir_name(&self) -> String {
        self.chain_name()
    }
}

impl Default for RoochChainID {
    fn default() -> Self {
        RoochChainID::Builtin(BuiltinChainID::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct RoochNetwork {
    pub chain_id: ChainID,
    pub genesis_config: GenesisConfig,
}

impl From<BuiltinChainID> for RoochNetwork {
    fn from(chain_id: BuiltinChainID) -> Self {
        RoochNetwork::builtin(chain_id)
    }
}

impl RoochNetwork {
    pub fn new(chain_id: ChainID, genesis_config: GenesisConfig) -> Self {
        Self {
            chain_id,
            genesis_config,
        }
    }

    pub fn builtin(builtin_id: BuiltinChainID) -> Self {
        Self::new(builtin_id.into(), builtin_id.genesis_config().clone())
    }

    pub fn local() -> Self {
        Self::builtin(BuiltinChainID::Local)
    }

    pub fn dev() -> Self {
        Self::builtin(BuiltinChainID::Dev)
    }

    pub fn test() -> Self {
        Self::builtin(BuiltinChainID::Test)
    }

    pub fn main() -> Self {
        Self::builtin(BuiltinChainID::Main)
    }

    pub fn set_sequencer_account(&mut self, account: AccountAddress) {
        self.genesis_config.sequencer_account = account;
    }
}
