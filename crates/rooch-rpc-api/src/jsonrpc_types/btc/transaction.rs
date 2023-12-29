// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use bitcoin::Txid;
use std::fmt;
use std::str::FromStr;

pub type TxidView = StrView<Txid>;

impl fmt::Display for TxidView {
    //TODO check display format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl FromStr for TxidView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(Txid::from_str(s)?))
    }
}

impl From<TxidView> for Txid {
    fn from(value: TxidView) -> Self {
        value.0
    }
}
