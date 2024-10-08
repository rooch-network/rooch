// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::{ArgGroup, Parser};
use moveos_types::h256::H256;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::ParsedAddress;
use rooch_types::authentication_key::AuthenticationKey;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::authenticator::Authenticator;
use rpassword::prompt_password;
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::Read;
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
            Ok(result) => {
                let output = serde_json::to_string_pretty(&result).unwrap();
                if output == "null" {
                    return Ok("".to_string());
                }
                Ok(output)
            }
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

    /// Custom account's sequence number
    #[clap(long)]
    pub(crate) sequence_number: Option<u64>,

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
#[clap(group = ArgGroup::new("filter").required(true).multiple(true))]
pub struct TransactionFilterOptions {
    /// Sender address
    #[clap(long, value_parser=ParsedAddress::parse, group = "filter")]
    pub(crate) sender: Option<ParsedAddress>,

    /// Transaction's hashes
    #[clap(long, value_delimiter = ',', group = "filter")]
    pub(crate) tx_hashes: Option<Vec<H256>>,

    /// [start-time, end-time) interval, unit: millisecond
    #[clap(long, requires = "end_time", group = "filter")]
    pub(crate) start_time: Option<u64>,
    /// [start-time, end-time) interval, unit: millisecond
    #[clap(long, requires = "start_time", group = "filter")]
    pub(crate) end_time: Option<u64>,

    /// [from-order, to-order) interval
    #[clap(long, requires = "to_order", group = "filter")]
    pub(crate) from_order: Option<u64>,
    /// [from-order, to-order) interval
    #[clap(long, requires = "from_order", group = "filter")]
    pub(crate) to_order: Option<u64>,
}

pub const ROOCH_PASSWORD_ENV: &str = "ROOCH_PASSWORD";

#[derive(Debug, Parser)]
pub struct WalletContextOptions {
    /// The key store password
    #[clap(long)]
    pub password: Option<String>,
    /// rooch config path
    #[clap(long)]
    pub config_dir: Option<PathBuf>,
}

impl WalletContextOptions {
    pub fn build(&self) -> RoochResult<WalletContext> {
        WalletContext::new(self.config_dir.clone()).map_err(RoochError::from)
    }

    pub fn build_require_password(&self) -> RoochResult<WalletContext> {
        let mut ctx = WalletContext::new(self.config_dir.clone()).map_err(RoochError::from)?;
        if ctx.keystore.get_if_password_is_empty() {
            Ok(ctx)
        } else {
            let password = self.password.clone().or_else(|| {
                //first try to get password from env
                //then prompt password
                match env::var(ROOCH_PASSWORD_ENV) {
                    Ok(val) => Some(val),
                    _ => {
                        let password = prompt_password("Enter the keystore password:").ok();
                        println!();
                        password
                    }
                }
            });
            let is_verified = verify_password(password.clone(), ctx.keystore.get_password_hash())?;
            if !is_verified {
                return Err(RoochError::InvalidPasswordError(
                    "Password is invalid".to_owned(),
                ));
            }
            ctx.set_password(password);
            Ok(ctx)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileOrHexInput {
    /// The data decode from file or hex string
    pub data: Vec<u8>,
}

impl FromStr for FileOrHexInput {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        let data_hex = if is_file_path(s) {
            //load hex from file
            let mut file = File::open(s)
                .map_err(|e| anyhow::anyhow!("Failed to open file: {}, err:{:?}", s, e))?;
            let mut hex_str = String::new();
            file.read_to_string(&mut hex_str)
                .map_err(|e| anyhow::anyhow!("Failed to read file: {}, err:{:?}", s, e))?;
            hex_str.strip_prefix("0x").unwrap_or(&hex_str).to_string()
        } else {
            s.strip_prefix("0x").unwrap_or(s).to_string()
        };
        let data = hex::decode(&data_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode hex: {}, err:{:?}", data_hex, e))?;
        Ok(FileOrHexInput { data })
    }
}

pub(crate) fn is_file_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.contains('.')
}
