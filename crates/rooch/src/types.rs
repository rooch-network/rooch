// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use rooch_client::wallet_context::WalletContext;
use rooch_types::error::{RoochError, RoochResult};
use serde::Serialize;
use std::path::PathBuf;

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

/// Common options for interacting with an account for a validator
#[derive(Debug, Default, Parser)]
pub struct TransactionOptions {
    /// Sender account address.
    /// This allows you to override the account address from the derived account address
    /// in the event that the authentication key was rotated or for a resource account
    #[clap(long)]
    pub(crate) sender_account: Option<String>,
}

#[derive(Debug, Parser)]
pub struct WalletContextOptions {
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
