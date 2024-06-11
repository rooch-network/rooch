// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use rooch_types::address::{BitcoinAddress, RoochAddress};
use std::str::FromStr;

pub type BitcoinAddressView = StrView<BitcoinAddress>;

impl std::fmt::Display for BitcoinAddressView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The Display Bitcoin address as a hexadecimal string
        write!(f, "{}", self.0)
    }
}

impl FromStr for BitcoinAddressView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(BitcoinAddress::from_str(s)?))
    }
}

impl From<BitcoinAddressView> for BitcoinAddress {
    fn from(value: BitcoinAddressView) -> Self {
        value.0
    }
}

pub type RoochAddressView = StrView<RoochAddress>;

impl std::fmt::Display for RoochAddressView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RoochAddressView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(RoochAddress::from_str(s)?))
    }
}

impl From<RoochAddressView> for RoochAddress {
    fn from(value: RoochAddressView) -> Self {
        value.0
    }
}

impl From<AccountAddress> for RoochAddressView {
    fn from(value: AccountAddress) -> Self {
        StrView(RoochAddress::from(value))
    }
}

#[derive(Debug, Clone)]
pub struct RoochOrBitcoinAddress {
    pub rooch_address: RoochAddress,
    pub bitcoin_address: Option<BitcoinAddress>,
}

pub type RoochOrBitcoinAddressView = StrView<RoochOrBitcoinAddress>;

impl std::fmt::Display for RoochOrBitcoinAddressView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.bitcoin_address.is_some() {
            return write!(f, "{}", self.0.bitcoin_address.as_ref().unwrap());
        }
        write!(f, "{}", self.0.rooch_address)
    }
}

impl FromStr for RoochOrBitcoinAddressView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match RoochAddress::from_str(s) {
            Ok(rooch_address) => Ok(StrView(RoochOrBitcoinAddress {
                rooch_address,
                bitcoin_address: None,
            })),
            Err(_) => {
                let bitcoin_address = BitcoinAddress::from_str(s)?;
                Ok(StrView(RoochOrBitcoinAddress {
                    rooch_address: bitcoin_address.to_rooch_address(),
                    bitcoin_address: Some(bitcoin_address),
                }))
            }
        }
    }
}

impl From<RoochOrBitcoinAddressView> for RoochAddress {
    fn from(value: RoochOrBitcoinAddressView) -> Self {
        value.0.rooch_address
    }
}

impl From<RoochAddressView> for RoochOrBitcoinAddressView {
    fn from(value: RoochAddressView) -> Self {
        StrView(RoochOrBitcoinAddress {
            rooch_address: value.into(),
            bitcoin_address: None,
        })
    }
}

impl TryFrom<RoochOrBitcoinAddressView> for BitcoinAddress {
    type Error = anyhow::Error;

    fn try_from(value: RoochOrBitcoinAddressView) -> Result<Self, Self::Error> {
        match value.0.bitcoin_address {
            Some(bitcoin_address) => Ok(bitcoin_address),
            None => Err(anyhow::anyhow!("No Bitcoin address found")),
        }
    }
}
