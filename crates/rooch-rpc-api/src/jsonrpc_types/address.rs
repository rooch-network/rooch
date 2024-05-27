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
