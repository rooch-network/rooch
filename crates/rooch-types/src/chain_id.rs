// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CHAIN_ID_DEV: u64 = 20230103;
pub const CHAIN_ID_TEST: u64 = 20230102;
pub const CHAIN_ID_MAIN: u64 = 20230101;

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Hash, Eq, PartialEq, PartialOrd, Ord, JsonSchema,
)]
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
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u64)]
pub enum BuiltinChainID {
    /// A ephemeral network just for developer test.
    Dev = CHAIN_ID_DEV,
    /// Rooch test network,
    /// The data on the chain will be cleaned up periodically.
    #[default]
    Test = CHAIN_ID_TEST,
    /// Rooch main net.
    Main = CHAIN_ID_MAIN,
}

impl Display for BuiltinChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinChainID::Dev => write!(f, "Dev"),
            BuiltinChainID::Test => write!(f, "Test"),
            BuiltinChainID::Main => write!(f, "Main"),
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

    pub fn is_dev(self) -> bool {
        matches!(self, BuiltinChainID::Dev)
    }

    pub fn is_test(self) -> bool {
        matches!(self, BuiltinChainID::Test)
    }

    pub fn assert_test_or_dev(self) -> Result<()> {
        if !self.is_test_or_dev() {
            bail!("Only support test or dev network.")
        }
        Ok(())
    }

    pub fn is_test_or_dev(self) -> bool {
        matches!(self, BuiltinChainID::Test | BuiltinChainID::Dev)
    }

    pub fn is_main(self) -> bool {
        matches!(self, BuiltinChainID::Main)
    }

    pub fn chain_ids() -> Vec<BuiltinChainID> {
        vec![
            BuiltinChainID::Dev,
            BuiltinChainID::Test,
            BuiltinChainID::Main,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
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

#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
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

impl FromStr for RoochChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match BuiltinChainID::from_str(s) {
            Ok(chain_id) => Ok(Self::Builtin(chain_id)),
            Err(_e) => Ok(Self::Custom(CustomChainID::from_str(s)?)),
        }
    }
}

// impl Serialize for RoochChainID {
//     fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(self.to_string().as_str())
//     }
// }
//
// impl<'de> Deserialize<'de> for RoochChainID {
//     fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = <String>::deserialize(deserializer)?;
//         Self::from_str(s.as_str()).map_err(D::Error::custom)
//     }
// }

impl RoochChainID {
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

    pub fn chain_id(&self) -> ChainID {
        match self {
            Self::Builtin(b) => b.chain_id(),
            Self::Custom(c) => c.chain_id(),
        }
    }

    pub fn assert_test_or_dev(&self) -> Result<()> {
        if !self.is_test_or_dev() {
            bail!("Only support test or dev chain_id.")
        }
        Ok(())
    }

    pub fn is_test_or_dev(&self) -> bool {
        self.is_test() || self.is_dev()
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
