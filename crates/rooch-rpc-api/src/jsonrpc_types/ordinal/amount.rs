// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, str::FromStr};

use bitcoin::Amount;

use crate::jsonrpc_types::StrView;

pub type AmountView = StrView<Amount>;

impl fmt::Display for AmountView {
    //TODO check display format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl FromStr for AmountView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(Amount::from_str(s)?))
    }
}

impl From<AmountView> for Amount {
    fn from(value: AmountView) -> Self {
        value.0
    }
}
