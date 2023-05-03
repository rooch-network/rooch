// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use std::str::FromStr;

/// Loads an account arg and allows for naming based on profiles
pub fn load_account_arg(str: &str) -> Result<AccountAddress> {
    if str.starts_with("0x") {
        AccountAddress::from_hex_literal(str)
            .map_err(|err| anyhow!("Failed to parse AccountAddress {}", err))
    } else if let Ok(account_address) = AccountAddress::from_str(str) {
        Ok(account_address)
    } else {
        bail!("'--account' must be provided".to_string(),)
    }
}

/// Common options for interacting with an account for a validator
#[derive(Debug, Default, Parser)]
pub struct TransactionOptions {
    /// Sender account address.
    /// This allows you to override the account address from the derived account address
    /// in the event that the authentication key was rotated or for a resource account
    #[clap(long, parse(try_from_str=load_account_arg))]
    pub(crate) sender_account: Option<AccountAddress>,
}
