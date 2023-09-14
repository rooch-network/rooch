// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::chain_id::{ChainID, RoochChainID};

pub const BITCOIN: u64 = 0;
pub const ETHEREUM: u64 = 60;
pub const NOSTR: u64 = 1237;

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Hash, Eq, PartialEq, PartialOrd, Ord, JsonSchema,
)]
pub struct MultiChainID {
    id: u64,
}

impl MultiChainID {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(self) -> u64 {
        self.id
    }

    pub fn is_ethereum(self) -> bool {
        self.id == ETHEREUM
    }

    pub fn is_bitcoin(self) -> bool {
        self.id == BITCOIN
    }

    pub fn is_nostr(self) -> bool {
        self.id == NOSTR
    }
}

impl Display for MultiChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for MultiChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u64 = s.parse()?;
        Ok(MultiChainID::new(id))
    }
}

impl From<u64> for MultiChainID {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<ChainID> for MultiChainID {
    fn from(chain_id: ChainID) -> Self {
        Self::new(chain_id.id())
    }
}

#[allow(clippy::from_over_into)]
impl Into<u64> for MultiChainID {
    fn into(self) -> u64 {
        self.id
    }
}

// BuiltinMultiChainID is following coin standard: https://github.com/satoshilabs/slips/blob/master/slip-0044.md
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[repr(u64)]
pub enum BuiltinMultiChainID {
    Bitcoin = BITCOIN,
    Ether = ETHEREUM,
    Nostr = NOSTR,
}

impl Display for BuiltinMultiChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinMultiChainID::Bitcoin => write!(f, "bitcoin"),
            BuiltinMultiChainID::Ether => write!(f, "ether"),
            BuiltinMultiChainID::Nostr => write!(f, "nostr"),
        }
    }
}

impl From<BuiltinMultiChainID> for u64 {
    fn from(multichain_id: BuiltinMultiChainID) -> Self {
        multichain_id as u64
    }
}

impl TryFrom<u64> for BuiltinMultiChainID {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            BITCOIN => Ok(BuiltinMultiChainID::Bitcoin),
            ETHEREUM => Ok(BuiltinMultiChainID::Ether),
            NOSTR => Ok(BuiltinMultiChainID::Nostr),
            _ => Err(anyhow::anyhow!("multichain id {} is invalid", value)),
        }
    }
}

impl FromStr for BuiltinMultiChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bitcoin" => Ok(BuiltinMultiChainID::Bitcoin),
            "ether" => Ok(BuiltinMultiChainID::Ether),
            "nostr" => Ok(BuiltinMultiChainID::Nostr),
            s => Err(format_err!("Unknown multichain: {}", s)),
        }
    }
}

impl TryFrom<MultiChainID> for BuiltinMultiChainID {
    type Error = anyhow::Error;
    fn try_from(multichain_id: MultiChainID) -> Result<Self, Self::Error> {
        Ok(match multichain_id.id() {
            BITCOIN => Self::Bitcoin,
            ETHEREUM => Self::Ether,
            NOSTR => Self::Nostr,
            id => bail!("{} is not a builtin multichain id", id),
        })
    }
}

impl BuiltinMultiChainID {
    pub fn multichain_name(self) -> String {
        self.to_string()
    }

    pub fn multichain_id(self) -> MultiChainID {
        MultiChainID::new(self.into())
    }

    pub fn is_bitcoin(self) -> bool {
        matches!(self, BuiltinMultiChainID::Bitcoin)
    }

    pub fn is_ethereum(self) -> bool {
        matches!(self, BuiltinMultiChainID::Ether)
    }

    pub fn is_nostr(self) -> bool {
        matches!(self, BuiltinMultiChainID::Nostr)
    }

    pub fn multichain_ids() -> Vec<BuiltinMultiChainID> {
        vec![
            BuiltinMultiChainID::Bitcoin,
            BuiltinMultiChainID::Ether,
            BuiltinMultiChainID::Nostr,
        ]
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema,
)]
#[allow(clippy::upper_case_acronyms)]
pub struct CustomMultiChainID {
    multichain_name: String,
    multichain_id: MultiChainID,
}

impl Display for CustomMultiChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.multichain_name, self.multichain_id)
    }
}

impl CustomMultiChainID {
    fn new(multichain_name: String, multichain_id: MultiChainID) -> Self {
        Self {
            multichain_name,
            multichain_id,
        }
    }

    pub fn multichain_id(&self) -> MultiChainID {
        self.multichain_id
    }

    pub fn multichain_name(&self) -> &str {
        self.multichain_name.as_str()
    }
}

impl FromStr for CustomMultiChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            bail!(
                "Invalid Custom multichain id {}, custom multichain id format is: multichain_name:multichain_id",
                s
            );
        }
        let multichain_name = parts[0].to_string();
        let multichain_id = MultiChainID::from_str(parts[1])?;
        Ok(Self::new(multichain_name, multichain_id))
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize, Deserialize,
)]
#[allow(clippy::upper_case_acronyms)]
pub enum RoochMultiChainID {
    Builtin(BuiltinMultiChainID),
    Custom(CustomMultiChainID),
}

impl Display for RoochMultiChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Builtin(b) => b.to_string(),
            Self::Custom(c) => c.to_string(),
        };
        write!(f, "{}", name)
    }
}

impl From<BuiltinMultiChainID> for RoochMultiChainID {
    fn from(multichain_id: BuiltinMultiChainID) -> Self {
        RoochMultiChainID::Builtin(multichain_id)
    }
}

impl From<CustomMultiChainID> for RoochMultiChainID {
    fn from(multichain_id: CustomMultiChainID) -> Self {
        RoochMultiChainID::Custom(multichain_id)
    }
}

impl TryFrom<MultiChainID> for RoochMultiChainID {
    type Error = anyhow::Error;

    fn try_from(multichain_id: MultiChainID) -> Result<Self, Self::Error> {
        Ok(match multichain_id.id() {
            BITCOIN => RoochMultiChainID::BITCOIN,
            ETHEREUM => RoochMultiChainID::ETHEREUM,
            NOSTR => RoochMultiChainID::NOSTR,
            id => RoochMultiChainID::Custom(CustomMultiChainID::from_str(id.to_string().as_str())?),
        })
    }
}

impl FromStr for RoochMultiChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match BuiltinMultiChainID::from_str(s) {
            Ok(multichain_id) => Ok(Self::Builtin(multichain_id)),
            Err(_e) => Ok(Self::Custom(CustomMultiChainID::from_str(s)?)),
        }
    }
}

impl RoochMultiChainID {
    pub const BITCOIN: RoochMultiChainID = RoochMultiChainID::Builtin(BuiltinMultiChainID::Bitcoin);
    pub const ETHEREUM: RoochMultiChainID = RoochMultiChainID::Builtin(BuiltinMultiChainID::Ether);
    pub const NOSTR: RoochMultiChainID = RoochMultiChainID::Builtin(BuiltinMultiChainID::Nostr);

    pub fn new_builtin(multichain_id: BuiltinMultiChainID) -> Self {
        Self::Builtin(multichain_id)
    }

    pub fn new_custom(multichain_name: String, multichain_id: MultiChainID) -> Result<Self> {
        for builtin_multichain_id in BuiltinMultiChainID::multichain_ids() {
            if builtin_multichain_id.multichain_id() == multichain_id {
                bail!(
                    "MultiChain id {} has used for builtin {}",
                    multichain_id,
                    builtin_multichain_id
                );
            }
            if builtin_multichain_id.multichain_name() == multichain_name {
                bail!(
                    "MultiChain name {} has used for builtin {}",
                    multichain_name,
                    builtin_multichain_id
                );
            }
        }
        Ok(Self::Custom(CustomMultiChainID::new(
            multichain_name,
            multichain_id,
        )))
    }

    pub fn multichain_id(&self) -> MultiChainID {
        match self {
            Self::Builtin(b) => b.multichain_id(),
            Self::Custom(c) => c.multichain_id(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        self.is_bitcoin() || self.is_ethereum() || self.is_nostr()
    }

    pub fn is_bitcoin(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinMultiChainID::Bitcoin))
    }

    pub fn is_ethereum(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinMultiChainID::Ether))
    }

    pub fn is_nostr(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinMultiChainID::Nostr))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Default data dir name of this multichain_id
    pub fn dir_name(&self) -> String {
        match self {
            Self::Builtin(b) => b.multichain_name().to_lowercase(),
            Self::Custom(c) => c.multichain_name().to_string().to_lowercase(),
        }
    }

    pub fn as_builtin(&self) -> Option<&BuiltinMultiChainID> {
        match self {
            Self::Builtin(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_custom(&self) -> Option<&CustomMultiChainID> {
        match self {
            Self::Custom(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_multichain(rooch_chain_id: &RoochChainID) -> Self {
        Self::new_custom(
            rooch_chain_id.chain_name(),
            MultiChainID::from(rooch_chain_id.chain_id()),
        )
        .unwrap()
    }
}
