// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u8)]
pub enum Network {
    NetworkBitcoin = 1,
    NetworkTestnet = 2,
    NetworkSignet = 3,
    NetworkRegtest = 4,
}

impl TryFrom<u8> for Network {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Network::NetworkBitcoin),
            2 => Ok(Network::NetworkTestnet),
            3 => Ok(Network::NetworkSignet),
            4 => Ok(Network::NetworkRegtest),
            _ => Err(anyhow::anyhow!("Bitcoin network {} is invalid", value)),
        }
    }
}

impl TryFrom<&str> for Network {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_uppercase();
        match value.as_str() {
            "bitcoin" => Ok(Network::NetworkBitcoin),
            "testnet" => Ok(Network::NetworkTestnet),
            "signet" => Ok(Network::NetworkSignet),
            "regtest" => Ok(Network::NetworkRegtest),
            _ => Err(anyhow::anyhow!("Bitcoin network {} is invalid", value)),
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::NetworkBitcoin => write!(f, "bitcoin"),
            Network::NetworkTestnet => write!(f, "testnet"),
            Network::NetworkSignet => write!(f, "signet"),
            Network::NetworkRegtest => write!(f, "regtest"),
        }
    }
}

impl Network {
    pub fn bech32_hrp(&self) -> bitcoin::bech32::Hrp {
        match self {
            Network::NetworkBitcoin => bitcoin::bech32::hrp::BC,
            Network::NetworkTestnet => bitcoin::bech32::hrp::TB,
            Network::NetworkSignet => bitcoin::bech32::hrp::TB,
            Network::NetworkRegtest => bitcoin::bech32::hrp::BCRT,
        }
    }

    pub fn to_num(self) -> u8 {
        self as u8
    }

    pub fn is_mainnet(&self) -> bool {
        *self == Network::NetworkBitcoin
    }
}

impl Default for Network {
    // default bitcoin regtest network
    fn default() -> Self {
        Self::NetworkBitcoin
    }
}

impl From<bitcoin::Network> for Network {
    fn from(network: bitcoin::Network) -> Self {
        match network {
            bitcoin::Network::Bitcoin => Self::NetworkBitcoin,
            bitcoin::Network::Testnet => Self::NetworkTestnet,
            bitcoin::Network::Signet => Self::NetworkSignet,
            bitcoin::Network::Regtest => Self::NetworkRegtest,
            _ => Self::NetworkRegtest,
        }
    }
}

impl From<Network> for bitcoin::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::NetworkBitcoin => bitcoin::Network::Bitcoin,
            Network::NetworkTestnet => bitcoin::Network::Testnet,
            Network::NetworkSignet => bitcoin::Network::Signet,
            Network::NetworkRegtest => bitcoin::Network::Regtest,
        }
    }
}
