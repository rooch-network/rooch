// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use crate::framework::genesis::GenesisContext;
use anyhow::{bail, format_err, Result};
use move_core_types::account_address::AccountAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const CHAIN_ID_LOCAL: u64 = 20230104;
pub const CHAIN_ID_DEV: u64 = 20230103;
pub const CHAIN_ID_TEST: u64 = 20230102;
pub const CHAIN_ID_MAIN: u64 = 20230101;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Hash, Eq, PartialEq, JsonSchema)]
pub struct ChainID {
    id: u64,
}

impl ChainID {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(self) -> u64 {
        self.id
    }

    pub fn is_local(self) -> bool {
        self.id == CHAIN_ID_LOCAL
    }

    pub fn is_dev(self) -> bool {
        self.id == CHAIN_ID_DEV
    }

    pub fn is_test(self) -> bool {
        self.id == CHAIN_ID_TEST
    }

    pub fn is_main(self) -> bool {
        self.id == CHAIN_ID_MAIN
    }
}

impl Display for ChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for ChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u64 = s.parse()?;
        Ok(ChainID::new(id))
    }
}

impl From<u64> for ChainID {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

#[allow(clippy::from_over_into)]
impl Into<u64> for ChainID {
    fn into(self) -> u64 {
        self.id
    }
}

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

impl BuiltinChainID {
    pub fn chain_name(self) -> String {
        self.to_string()
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

    pub fn genesis_ctx(&self, sequencer: RoochAddress, gas_schedule_blob: Vec<u8>) -> GenesisContext {
        let chain_id = self.chain_id().id;
        let sequencer_account = AccountAddress::from(sequencer);
        match self {
            BuiltinChainID::Local => {
                //Local timestamp from 0, developer can manually set the timestamp
                let timestamp = 0;
                GenesisContext::new(chain_id, timestamp, sequencer_account, gas_schedule_blob)
            }
            BuiltinChainID::Dev => {
                //Dev network start from Ethereum block height 9685149, timestamp: 1694571540
                let timestamp = std::time::Duration::from_secs(1694571540).as_micros() as u64;
                GenesisContext::new(chain_id, timestamp, sequencer_account, gas_schedule_blob)
            }
            BuiltinChainID::Test => {
                //Test network start from Ethereum block height 9685149, timestamp: 1694571540
                let timestamp = std::time::Duration::from_secs(1694571540).as_micros() as u64;
                GenesisContext::new(chain_id, timestamp, sequencer_account, gas_schedule_blob)
            }
            BuiltinChainID::Main => {
                //TODO config main network genesis timestamp
                let timestamp = 0;
                GenesisContext::new(chain_id, timestamp, sequencer_account, gas_schedule_blob)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct CustomChainID {
    chain_name: String,
    chain_id: ChainID,
}

impl Display for CustomChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.chain_name, self.chain_id)
    }
}

impl CustomChainID {
    fn new(chain_name: String, chain_id: ChainID) -> Self {
        Self {
            chain_name,
            chain_id,
        }
    }

    pub fn chain_id(&self) -> ChainID {
        self.chain_id
    }

    pub fn chain_name(&self) -> &str {
        self.chain_name.as_str()
    }

    pub fn genesis_ctx(&self, sequencer: RoochAddress, gas_schedule_blob: Vec<u8>) -> GenesisContext {
        //TODO support custom chain genesis timestamp
        let timestamp = 0;
        let sequencer_account = AccountAddress::from(sequencer);
        GenesisContext::new(self.chain_id.id, timestamp, sequencer_account, gas_schedule_blob)
    }
}

impl FromStr for CustomChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            bail!(
                "Invalid Custom chain id {}, custom chain id format is: chain_name:chain_id",
                s
            );
        }
        let chain_name = parts[0].to_string();
        let chain_id = ChainID::from_str(parts[1])?;
        Ok(Self::new(chain_name, chain_id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, JsonSchema, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum RoochChainID {
    Builtin(BuiltinChainID),
    Custom(CustomChainID),
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

impl From<CustomChainID> for RoochChainID {
    fn from(chain_id: CustomChainID) -> Self {
        RoochChainID::Custom(chain_id)
    }
}

impl TryFrom<ChainID> for RoochChainID {
    type Error = anyhow::Error;

    fn try_from(chain_id: ChainID) -> Result<Self, Self::Error> {
        Ok(match chain_id.id() {
            CHAIN_ID_LOCAL => RoochChainID::LOCAL,
            CHAIN_ID_DEV => RoochChainID::DEV,
            CHAIN_ID_TEST => RoochChainID::TEST,
            CHAIN_ID_MAIN => RoochChainID::MAIN,
            id => RoochChainID::Custom(CustomChainID::from_str(id.to_string().as_str())?),
        })
    }
}

impl FromStr for RoochChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match BuiltinChainID::from_str(s) {
            Ok(chain_id) => Ok(Self::Builtin(chain_id)),
            Err(_e) => Ok(Self::Custom(CustomChainID::from_str(s)?)),
        }
    }
}

impl RoochChainID {
    pub const LOCAL: RoochChainID = RoochChainID::Builtin(BuiltinChainID::Local);
    pub const DEV: RoochChainID = RoochChainID::Builtin(BuiltinChainID::Dev);
    pub const TEST: RoochChainID = RoochChainID::Builtin(BuiltinChainID::Test);
    pub const MAIN: RoochChainID = RoochChainID::Builtin(BuiltinChainID::Main);

    pub fn new_builtin(chain_id: BuiltinChainID) -> Self {
        Self::Builtin(chain_id)
    }

    pub fn new_custom(chain_name: String, chain_id: ChainID) -> Result<Self> {
        for builtin_chain_id in BuiltinChainID::chain_ids() {
            if builtin_chain_id.chain_id() == chain_id {
                bail!(
                    "Chain id {} has used for builtin {}",
                    chain_id,
                    builtin_chain_id
                );
            }
            if builtin_chain_id.chain_name() == chain_name {
                bail!(
                    "Chain name {} has used for builtin {}",
                    chain_name,
                    builtin_chain_id
                );
            }
        }
        Ok(Self::Custom(CustomChainID::new(chain_name, chain_id)))
    }

    pub fn chain_name(&self) -> String {
        match self {
            Self::Builtin(b) => b.chain_name(),
            Self::Custom(c) => c.chain_name().to_owned(),
        }
    }

    pub fn chain_id(&self) -> ChainID {
        match self {
            Self::Builtin(b) => b.chain_id(),
            Self::Custom(c) => c.chain_id(),
        }
    }

    pub fn genesis_ctx(&self, sequencer: RoochAddress, gas_schedule_blob: Vec<u8>) -> GenesisContext {
        match self {
            Self::Builtin(b) => b.genesis_ctx(sequencer, gas_schedule_blob),
            Self::Custom(c) => c.genesis_ctx(sequencer, gas_schedule_blob),
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
        match self {
            Self::Builtin(b) => b.chain_name().to_lowercase(),
            Self::Custom(c) => c.chain_name().to_string().to_lowercase(),
        }
    }

    pub fn as_builtin(&self) -> Option<&BuiltinChainID> {
        match self {
            Self::Builtin(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_custom(&self) -> Option<&CustomChainID> {
        match self {
            Self::Custom(c) => Some(c),
            _ => None,
        }
    }
}

impl Default for RoochChainID {
    fn default() -> Self {
        RoochChainID::Builtin(BuiltinChainID::default())
    }
}
