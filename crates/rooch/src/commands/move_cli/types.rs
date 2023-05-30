// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_common::config::{rooch_config_path, PersistedConfig, RoochConfig};
use rooch_types::cli::{CliError, CliResult};
use std::str::FromStr;

/// A wrapper around `AccountAddress` to be more flexible from strings than AccountAddress
#[derive(Clone, Copy, Debug)]
pub struct AccountAddressWrapper {
    pub account_address: AccountAddress,
}

impl FromStr for AccountAddressWrapper {
    type Err = CliError;
    fn from_str(s: &str) -> CliResult<Self> {
        Ok(AccountAddressWrapper {
            account_address: load_account_arg(s)?,
        })
    }
}

/// Loads an account arg and allows for naming based on profiles
pub fn load_account_arg(str: &str) -> CliResult<AccountAddress> {
    if str.starts_with("0x") {
        AccountAddress::from_hex_literal(str).map_err(|err| {
            CliError::CommandArgumentError(format!("Failed to parse AccountAddress {}", err))
        })
    } else if let Ok(account_address) = AccountAddress::from_str(str) {
        Ok(account_address)
    } else {
        let config: RoochConfig =
            PersistedConfig::read(rooch_config_path().map_err(CliError::from)?.as_path()).map_err(
                |_| CliError::UnexpectedError(format!("{:?}", "Use `rooch init` to configuration")),
            )?;

        let address = match str {
            "default" => AccountAddress::new(config.active_address.unwrap().0.into()),
            _ => Err(CliError::CommandArgumentError(
                "Use rooch init configuration".to_string(),
            ))?,
        };

        Ok(address)
    }
}

/// Common options for interacting with an account for a validator
#[derive(Debug, Parser)]
pub struct TransactionOptions {
    /// Sender account address.
    /// This allows you to override the account address from the derived account address
    /// in the event that the authentication key was rotated or for a resource account
    #[clap(long, parse(try_from_str=load_account_arg), default_value="default")]
    pub(crate) sender_account: AccountAddress,
}
