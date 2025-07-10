// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::commands::init::InitCommand;
use self::commands::open::OpenCommand;
use self::commands::create_rav::CreateRavCommand;
use self::commands::claim::ClaimCommand;
use self::commands::close::CloseCommand;
use self::commands::cancel::CancelCommand;
use self::commands::dispute::DisputeCommand;
use self::commands::finalize_cancellation::FinalizeCancellationCommand;
use self::commands::query::QueryCommand;
use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;
use serde_json::Value;

pub mod commands;

#[derive(Parser)]
#[clap(about = "Payment Channel management commands")]
pub struct PaymentChannel {
    #[clap(subcommand)]
    cmd: PaymentChannelCommand,
}

#[async_trait]
impl CommandAction<String> for PaymentChannel {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            PaymentChannelCommand::Init(init) => {
                let resp = init.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::Open(open) => {
                let resp = open.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::CreateRav(create_rav) => {
                let resp = create_rav.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::Claim(claim) => {
                let resp = claim.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::Close(close) => {
                let resp = close.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::Cancel(cancel) => {
                let resp = cancel.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::Dispute(dispute) => {
                let resp = dispute.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::FinalizeCancellation(finalize) => {
                let resp = finalize.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            PaymentChannelCommand::Query(query) => {
                let json_output = query.execute_serialized().await?;
                let json_value: Value = serde_json::from_str(&json_output)?;
                Ok(serde_json::to_string_pretty(&json_value)?)
            }
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "payment-channel")]
pub enum PaymentChannelCommand {
    /// Initialize payment hub and deposit tokens
    #[clap(name = "init")]
    Init(InitCommand),

    /// Open a payment channel with sub-channels
    #[clap(name = "open")]
    Open(OpenCommand),

    /// Create a signed SubRAV for off-chain payment
    #[clap(name = "create-rav")]
    CreateRav(CreateRavCommand),

    /// Claim funds from a channel using SubRAV
    #[clap(name = "claim")]
    Claim(ClaimCommand),

    /// Close a channel with final settlement
    #[clap(name = "close")]
    Close(CloseCommand),

    /// Initiate channel cancellation
    #[clap(name = "cancel")]
    Cancel(CancelCommand),

    /// Dispute a cancellation with newer state
    #[clap(name = "dispute")]
    Dispute(DisputeCommand),

    /// Finalize cancellation after challenge period
    #[clap(name = "finalize-cancellation")]
    FinalizeCancellation(FinalizeCancellationCommand),

    /// Query payment hub or channel state
    #[clap(name = "query")]
    Query(QueryCommand),
} 