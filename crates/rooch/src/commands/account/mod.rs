// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod create_account;
use crate::commands::account::create_account::CreateAccount;
use anyhow::Result;

#[derive(Debug, clap::Subcommand)]
#[clap(name = "account")]
pub enum AccountCommand {
    Create(CreateAccount),
    // CreateResourceAccount(create_resource_account::CreateResourceAccount),
    // List(list::ListAccount),
    // RotateKey(key_rotation::RotateKey),
}

impl AccountCommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            AccountCommand::Create(c) => c.execute().await,
            // AccountCommand::CreateResourceAccount(c) => c.execute_serialized().await,
            // AccountCommand::List(c) => c.execute_serialized().await,
            // AccountCommand::RotateKey(c) => c.execute_serialized().await,
        }
    }
}
