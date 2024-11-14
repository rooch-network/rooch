// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use commands::{
    balance::BalanceCommand, create::CreateCommand, create_multisign::CreateMultisignCommand,
    export::ExportCommand, import::ImportCommand, list::ListCommand, nullify::NullifyCommand,
    sign::SignCommand, switch::SwitchCommand, transfer::TransferCommand, verify::VerifyCommand,
};
use rooch_rpc_api::jsonrpc_types::json_to_table_display::json_to_table;
use rooch_types::error::RoochResult;
use serde_json::Value;
use std::path::PathBuf;

pub mod commands;
/// Tool for interacting with accounts
#[derive(clap::Parser)]
pub struct Account {
    #[clap(subcommand)]
    cmd: AccountCommand,
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "client.config")]
    config: Option<PathBuf>,
}

#[async_trait]
impl CommandAction<String> for Account {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            AccountCommand::Create(create) => create.execute_serialized().await,
            AccountCommand::CreateMultisign(create_multisign) => {
                create_multisign.execute_serialized().await
            }
            AccountCommand::List(list) => list.execute_serialized().await,
            AccountCommand::Switch(switch) => switch.execute_serialized().await,
            AccountCommand::Nullify(nullify) => nullify.execute_serialized().await,
            AccountCommand::Balance(balance) => balance.execute_serialized().await,
            AccountCommand::Transfer(transfer) => {
                let output_as_json = transfer.json;
                let output = transfer.execute_serialized().await?;
                if output_as_json {
                    Ok(output)
                } else if let Ok(json_value) = serde_json::from_str::<Value>(&output) {
                    json_to_table(json_value);
                    Ok(String::new())
                } else {
                    Ok(output)
                }
            }
            AccountCommand::Export(export) => export.execute_serialized().await,
            AccountCommand::Import(import) => import.execute_serialized().await,
            AccountCommand::Sign(sign) => sign.execute_serialized().await,
            AccountCommand::Verify(verify) => verify.execute_serialized().await,
        }
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "account")]
pub enum AccountCommand {
    Create(CreateCommand),
    CreateMultisign(CreateMultisignCommand),
    List(ListCommand),
    Switch(SwitchCommand),
    Nullify(NullifyCommand),
    Balance(BalanceCommand),
    Transfer(TransferCommand),
    Export(ExportCommand),
    Import(ImportCommand),
    Sign(SignCommand),
    Verify(VerifyCommand),
}
