// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use move_core_types::transaction_argument::TransactionArgument;
use move_core_types::u256::U256;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::authentication_key::AuthenticationKey;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::authenticator::Authenticator;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;

use std::str::FromStr;

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

#[derive(Debug)]
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
#[derive(Debug, Default, Parser)]
pub struct TransactionOptions {
    /// Sender account address.
    /// This allows you to override the account address from the derived account address
    /// in the event that the authentication key was rotated or for a resource account
    //TODO set default value to sender account
    #[clap(long, alias = "sender")]
    pub(crate) sender_account: Option<String>,

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
    pub async fn build(&self) -> RoochResult<WalletContext> {
        WalletContext::new(self.config_dir.clone())
            .await
            .map_err(RoochError::from)
    }
}

/// A wrapper around `AccountAddress` to be more flexible from strings than AccountAddress
#[derive(Clone, Copy, Debug)]
pub struct AccountAddressWrapper {
    pub account_address: AccountAddress,
}

impl FromStr for AccountAddressWrapper {
    type Err = RoochError;
    fn from_str(s: &str) -> RoochResult<Self> {
        Ok(AccountAddressWrapper {
            account_address: load_account_arg(s)?,
        })
    }
}

/// Loads an account arg and allows for naming based on profiles
pub fn load_account_arg(str: &str) -> RoochResult<AccountAddress> {
    if str.starts_with("0x") {
        AccountAddress::from_hex_literal(str).map_err(|err| {
            RoochError::CommandArgumentError(format!("Failed to parse AccountAddress {}", err))
        })
    } else if let Ok(account_address) = AccountAddress::from_str(str) {
        Ok(account_address)
    } else {
        Err(RoochError::UnableToParse(
            "Address",
            "Address should be in hex format".to_owned(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FunctionArgType {
    Address,
    Bool,
    ObjectID,
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Raw,
    Vector(Box<FunctionArgType>),
}

impl Display for FunctionArgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionArgType::Address => write!(f, "address"),
            FunctionArgType::Bool => write!(f, "bool"),
            FunctionArgType::ObjectID => write!(f, "object_id"),
            FunctionArgType::String => write!(f, "string"),
            FunctionArgType::U8 => write!(f, "u8"),
            FunctionArgType::U16 => write!(f, "u16"),
            FunctionArgType::U32 => write!(f, "u32"),
            FunctionArgType::U64 => write!(f, "u64"),
            FunctionArgType::U128 => write!(f, "u128"),
            FunctionArgType::U256 => write!(f, "u256"),
            FunctionArgType::Raw => write!(f, "raw"),
            FunctionArgType::Vector(inner) => write!(f, "vector<{}>", inner),
        }
    }
}

impl FunctionArgType {
    fn parse_arg(&self, arg: &str) -> RoochResult<Vec<u8>> {
        match self {
            FunctionArgType::Address => bcs::to_bytes(
                &load_account_arg(arg)
                    .map_err(|err| RoochError::UnableToParse("address", err.to_string()))?,
            ),
            FunctionArgType::Bool => bcs::to_bytes(
                &bool::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("bool", err.to_string()))?,
            ),
            FunctionArgType::ObjectID => bcs::to_bytes(
                &load_account_arg(arg)
                    .map_err(|err| RoochError::UnableToParse("object_id", err.to_string()))?,
            ),
            FunctionArgType::String => bcs::to_bytes(arg),
            FunctionArgType::U8 => bcs::to_bytes(
                &u8::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("u8", err.to_string()))?,
            ),
            FunctionArgType::U16 => bcs::to_bytes(
                &u16::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("u16", err.to_string()))?,
            ),
            FunctionArgType::U32 => bcs::to_bytes(
                &u32::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("u32", err.to_string()))?,
            ),
            FunctionArgType::U64 => bcs::to_bytes(
                &u64::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("u64", err.to_string()))?,
            ),
            FunctionArgType::U128 => bcs::to_bytes(
                &u128::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("u128", err.to_string()))?,
            ),
            FunctionArgType::U256 => bcs::to_bytes(
                &U256::from_str(arg)
                    .map_err(|err| RoochError::UnableToParse("u256", err.to_string()))?,
            ),
            FunctionArgType::Raw => {
                let raw = hex::decode(arg)
                    .map_err(|err| RoochError::UnableToParse("raw", err.to_string()))?;
                Ok(raw)
            }
            FunctionArgType::Vector(inner) => {
                let parsed = match inner.deref() {
                    FunctionArgType::Address => parse_vector_arg(arg, |arg| {
                        load_account_arg(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<address>", err.to_string())
                        })
                    }),
                    FunctionArgType::Bool => parse_vector_arg(arg, |arg| {
                        bool::from_str(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<bool>", err.to_string())
                        })
                    }),
                    FunctionArgType::ObjectID => parse_vector_arg(arg, |arg| {
                        load_account_arg(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<object_id>", err.to_string())
                        })
                    }),
                    // Note commas cannot be put into the strings.  But, this should be a less likely case,
                    // and the utility from having this available should be worth it.
                    FunctionArgType::String => parse_vector_arg(arg, |arg| {
                        bcs::to_bytes(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<string>", err.to_string())
                        })
                    }),
                    FunctionArgType::U8 => parse_vector_arg(arg, |arg| {
                        u8::from_str(arg)
                            .map_err(|err| RoochError::UnableToParse("vector<u8>", err.to_string()))
                    }),
                    FunctionArgType::U16 => parse_vector_arg(arg, |arg| {
                        u16::from_str(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<u16>", err.to_string())
                        })
                    }),
                    FunctionArgType::U32 => parse_vector_arg(arg, |arg| {
                        u32::from_str(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<u32>", err.to_string())
                        })
                    }),
                    FunctionArgType::U64 => parse_vector_arg(arg, |arg| {
                        u64::from_str(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<u64>", err.to_string())
                        })
                    }),
                    FunctionArgType::U128 => parse_vector_arg(arg, |arg| {
                        u128::from_str(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<u128>", err.to_string())
                        })
                    }),
                    FunctionArgType::U256 => parse_vector_arg(arg, |arg| {
                        U256::from_str(arg).map_err(|err| {
                            RoochError::UnableToParse("vector<u256>", err.to_string())
                        })
                    }),
                    vector_type => {
                        panic!("Unsupported vector type vector<{}>", vector_type)
                    }
                }?;
                Ok(parsed)
            }
        }
        .map_err(|err| RoochError::BcsError(err.to_string()))
    }
}

fn parse_vector_arg<T: Serialize, F: Fn(&str) -> RoochResult<T>>(
    args: &str,
    parse: F,
) -> RoochResult<Vec<u8>> {
    let mut parsed_args = vec![];
    let args = args.split(',');
    for arg in args {
        if !arg.is_empty() {
            parsed_args.push(parse(arg)?);
        }
    }

    bcs::to_bytes(&parsed_args).map_err(|err| RoochError::BcsError(err.to_string()))
}

impl FromStr for FunctionArgType {
    type Err = RoochError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "address" => Ok(FunctionArgType::Address),
            "bool" => Ok(FunctionArgType::Bool),
            "object_id" => Ok(FunctionArgType::ObjectID),
            "string" => Ok(FunctionArgType::String),
            "u8" => Ok(FunctionArgType::U8),
            "u16" => Ok(FunctionArgType::U16),
            "u32" => Ok(FunctionArgType::U32),
            "u64" => Ok(FunctionArgType::U64),
            "u128" => Ok(FunctionArgType::U128),
            "u256" => Ok(FunctionArgType::U256),
            "raw" => Ok(FunctionArgType::Raw),
            str => {
                // If it's a vector, go one level inside
                if str.starts_with("vector<") && str.ends_with('>') {
                    let arg = FunctionArgType::from_str(&str[7..str.len() - 1])?;

                    if arg == FunctionArgType::Raw {
                        return Err(RoochError::CommandArgumentError(
                            "vector<raw> is not supported".to_owned(),
                        ));
                    } else if matches!(arg, FunctionArgType::Vector(_)) {
                        return Err(RoochError::CommandArgumentError(
                            "nested vector<vector<_>> is not supported".to_owned(),
                        ));
                    }

                    Ok(FunctionArgType::Vector(Box::new(arg)))
                } else {
                    Err(RoochError::CommandArgumentError(format!("Invalid arg type '{}'.  Must be one of: ['address','bool','object_id','string','u8','u16','u32','u64','u128','u256','vector<inner_type>']", str)))
                }
            }
        }
    }
}

/// A parseable arg with a type separated by a colon
pub struct ArgWithType {
    pub(crate) _ty: FunctionArgType,
    pub(crate) arg: Vec<u8>,
}

impl ArgWithType {
    pub fn address(account_address: AccountAddress) -> Self {
        ArgWithType {
            _ty: FunctionArgType::Address,
            arg: bcs::to_bytes(&account_address).unwrap(),
        }
    }

    pub fn u64(arg: u64) -> Self {
        ArgWithType {
            _ty: FunctionArgType::U64,
            arg: bcs::to_bytes(&arg).unwrap(),
        }
    }

    pub fn bytes(arg: Vec<u8>) -> Self {
        ArgWithType {
            _ty: FunctionArgType::Raw,
            arg: bcs::to_bytes(&arg).unwrap(),
        }
    }

    pub fn raw(arg: Vec<u8>) -> Self {
        ArgWithType {
            _ty: FunctionArgType::Raw,
            arg,
        }
    }

    pub fn to_json(&self) -> RoochResult<serde_json::Value> {
        match self._ty.clone() {
            FunctionArgType::Address => {
                serde_json::to_value(bcs::from_bytes::<AccountAddress>(&self.arg)?)
            }
            FunctionArgType::Bool => serde_json::to_value(bcs::from_bytes::<bool>(&self.arg)?),
            FunctionArgType::ObjectID => {
                serde_json::to_value(bcs::from_bytes::<Vec<u8>>(&self.arg)?)
            }
            FunctionArgType::String => serde_json::to_value(bcs::from_bytes::<String>(&self.arg)?),
            FunctionArgType::U8 => serde_json::to_value(bcs::from_bytes::<u32>(&self.arg)?),
            FunctionArgType::U16 => serde_json::to_value(bcs::from_bytes::<u32>(&self.arg)?),
            FunctionArgType::U32 => serde_json::to_value(bcs::from_bytes::<u32>(&self.arg)?),
            FunctionArgType::U64 => {
                serde_json::to_value(bcs::from_bytes::<u64>(&self.arg)?.to_string())
            }
            FunctionArgType::U128 => {
                serde_json::to_value(bcs::from_bytes::<u128>(&self.arg)?.to_string())
            }
            FunctionArgType::U256 => {
                serde_json::to_value(bcs::from_bytes::<U256>(&self.arg)?.to_string())
            }
            FunctionArgType::Raw => serde_json::to_value(&self.arg),
            FunctionArgType::Vector(inner) => match inner.deref() {
                FunctionArgType::Address => {
                    serde_json::to_value(bcs::from_bytes::<Vec<AccountAddress>>(&self.arg)?)
                }
                FunctionArgType::Bool => {
                    serde_json::to_value(bcs::from_bytes::<Vec<bool>>(&self.arg)?)
                }
                FunctionArgType::ObjectID => {
                    serde_json::to_value(bcs::from_bytes::<Vec<Vec<u8>>>(&self.arg)?)
                }
                FunctionArgType::String => {
                    serde_json::to_value(bcs::from_bytes::<Vec<String>>(&self.arg)?)
                }
                FunctionArgType::U8 => serde_json::to_value(bcs::from_bytes::<Vec<u8>>(&self.arg)?),
                FunctionArgType::U16 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u16>>(&self.arg)?)
                }
                FunctionArgType::U32 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u32>>(&self.arg)?)
                }
                FunctionArgType::U64 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u64>>(&self.arg)?)
                }
                FunctionArgType::U128 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u128>>(&self.arg)?)
                }
                FunctionArgType::U256 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<U256>>(&self.arg)?)
                }
                FunctionArgType::Raw | FunctionArgType::Vector(_) => {
                    return Err(RoochError::UnexpectedError(
                        "Nested vectors not supported".to_owned(),
                    ));
                }
            },
        }
        .map_err(|err| {
            RoochError::UnexpectedError(format!("Failed to parse argument to JSON {}", err))
        })
    }
}

impl FromStr for ArgWithType {
    type Err = RoochError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Splits on the first colon, returning at most `2` elements
        // This is required to support args that contain a colon
        let parts: Vec<_> = s.splitn(2, ':').collect();
        let (ty, arg) = if parts.len() == 1 {
            // parse address @0x123 and unsigned integer 123u8
            if s.starts_with('@') {
                (FunctionArgType::Address, s.trim_start_matches('@'))
            } else {
                let u = s.splitn(2, 'u').collect::<Vec<_>>();
                if u.len() != 2 {
                    return Err(RoochError::CommandArgumentError(
                        "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_owned(),
                    ));
                } else {
                    let ty_str = String::from("u") + u[1];
                    let ty = FunctionArgType::from_str(&ty_str)?;
                    let arg = u[0];
                    (ty, arg)
                }
            }
        } else if parts.len() == 2 {
            let ty = FunctionArgType::from_str(parts[0])?;
            let arg = parts[1];
            (ty, arg)
        } else {
            return Err(RoochError::CommandArgumentError(
                "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_owned(),
            ));
        };
        let arg = ty.parse_arg(arg)?;

        Ok(ArgWithType { _ty: ty, arg })
    }
}

impl TryInto<TransactionArgument> for ArgWithType {
    type Error = RoochError;

    fn try_into(self) -> Result<TransactionArgument, Self::Error> {
        match self._ty {
            FunctionArgType::Address => Ok(TransactionArgument::Address(txn_arg_parser(
                &self.arg, "address",
            )?)),
            FunctionArgType::Bool => Ok(TransactionArgument::Bool(txn_arg_parser(
                &self.arg, "bool",
            )?)),
            FunctionArgType::ObjectID => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg,
                "object_id",
            )?)),
            FunctionArgType::String => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg, "string",
            )?)),
            FunctionArgType::U8 => Ok(TransactionArgument::U8(txn_arg_parser(&self.arg, "u8")?)),
            FunctionArgType::U16 => Ok(TransactionArgument::U16(txn_arg_parser(&self.arg, "u16")?)),
            FunctionArgType::U32 => Ok(TransactionArgument::U32(txn_arg_parser(&self.arg, "u32")?)),
            FunctionArgType::U64 => Ok(TransactionArgument::U64(txn_arg_parser(&self.arg, "u64")?)),
            FunctionArgType::U128 => Ok(TransactionArgument::U128(txn_arg_parser(
                &self.arg, "u128",
            )?)),
            FunctionArgType::U256 => Ok(TransactionArgument::U256(txn_arg_parser(
                &self.arg, "u256",
            )?)),
            FunctionArgType::Raw => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg, "raw",
            )?)),
            arg_type => Err(RoochError::CommandArgumentError(format!(
                "Input type {} not supported",
                arg_type
            ))),
        }
    }
}

fn txn_arg_parser<T: serde::de::DeserializeOwned>(
    data: &[u8],
    label: &'static str,
) -> Result<T, RoochError> {
    bcs::from_bytes(data).map_err(|err| RoochError::UnableToParse(label, err.to_string()))
}
