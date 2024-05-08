// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::address::ParsedAddress;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::authentication_key::AuthenticationKey;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::authenticator::Authenticator;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

pub use rooch_types::function_arg::{FunctionArg, FunctionArgType};

#[async_trait]
pub trait CommandAction<T: Serialize + Send>: Sized + Send {
    /// Executes the command, returning a command specific type
    async fn execute(self) -> RoochResult<T>;

    /// Executes the command, and serializes it to the common JSON output type
    async fn execute_serialized(self) -> RoochResult<String> {
        match self.execute().await {
            Ok(result) => Ok(serde_json::to_string_pretty(&result).unwrap()),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatorOptions {
    pub auth_validator_id: u64,
    pub payload: Vec<u8>,
}

impl FromStr for AuthenticatorOptions {
    type Err = RoochError;
    fn from_str(s: &str) -> RoochResult<Self> {
        let mut split = s.split(':');
        let auth_validator_id = split.next().ok_or_else(|| {
            RoochError::CommandArgumentError(format!("Invalid authenticator argument: {}", s))
        })?;
        let auth_validator_id = auth_validator_id.parse::<u64>().map_err(|_| {
            RoochError::CommandArgumentError(format!("Invalid authenticator argument: {}", s))
        })?;
        let payload = split.next().ok_or_else(|| {
            RoochError::CommandArgumentError(format!("Invalid authenticator argument: {}", s))
        })?;
        let payload = hex::decode(payload.strip_prefix("0x").unwrap_or(payload)).map_err(|_| {
            RoochError::CommandArgumentError(format!("Invalid authenticator argument: {}", s))
        })?;
        Ok(AuthenticatorOptions {
            auth_validator_id,
            payload,
        })
    }
}

impl From<AuthenticatorOptions> for Authenticator {
    fn from(options: AuthenticatorOptions) -> Self {
        Authenticator {
            auth_validator_id: options.auth_validator_id,
            payload: options.payload,
        }
    }
}

/// Common options for interacting with an account for a validator
#[derive(Debug, Parser)]
pub struct TransactionOptions {
    /// Sender account address.
    #[clap(long, alias = "sender-account", value_parser=ParsedAddress::parse, default_value = "default")]
    pub(crate) sender: ParsedAddress,

    /// Custom the transaction's gas limit.
    /// [default: 1_000_000_000] [alias: "gas-limit"]
    #[clap(long, alias = "gas-limit")]
    pub(crate) max_gas_amount: Option<u64>,

    /// Custom the transaction's authenticator
    /// format: `auth_validator_id:payload`, auth validator id is u64, payload is hex string
    /// example: 123:0x2abc
    #[clap(long)]
    pub(crate) authenticator: Option<AuthenticatorOptions>,

    /// Sign the transaction via session key
    /// This option conflicts with `authenticator`
    #[clap(long, conflicts_with = "authenticator")]
    pub(crate) session_key: Option<AuthenticationKey>,
}

#[derive(Debug, Parser)]
pub struct WalletContextOptions {
    /// rooch config path
    #[clap(long)]
    pub config_dir: Option<PathBuf>,
}

impl WalletContextOptions {
    pub fn build(&self) -> RoochResult<WalletContext> {
        WalletContext::new(self.config_dir.clone()).map_err(RoochError::from)
    }
}
