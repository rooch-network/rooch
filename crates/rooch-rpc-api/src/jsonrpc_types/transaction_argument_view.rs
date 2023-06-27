// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use move_core_types::{
    parser::parse_transaction_argument, transaction_argument::TransactionArgument, value::MoveValue,
};
use std::str::FromStr;

pub type TransactionArgumentView = StrView<TransactionArgument>;

impl std::fmt::Display for StrView<TransactionArgument> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", MoveValue::from(self.0.clone()))
    }
}

impl FromStr for TransactionArgumentView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //TODO support more argument types
        // vector<address>, vector<std::string::String>, etc.
        let arg = parse_transaction_argument(s)?;
        Ok(Self(arg))
    }
}
