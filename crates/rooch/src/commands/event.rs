// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use move_core_types::language_storage::StructTag;
use rooch_rpc_api::jsonrpc_types::EventPageViewResult;
use rooch_types::error::{RoochError, RoochResult};

/// Tool for interacting with event
#[derive(Parser)]
pub struct EventCommand {
    #[clap(subcommand)]
    cmd: EventSubCommand,
}

#[async_trait]
impl CommandAction<String> for EventCommand {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            EventSubCommand::GetEventsByEventHandle(cmd) => cmd.execute_serialized().await,
        }
    }
}

#[derive(Subcommand)]
pub enum EventSubCommand {
    GetEventsByEventHandle(GetEventsByEventHandle),
}
/// Retrieves events based on their event handle.
#[derive(Debug, Parser)]
pub struct GetEventsByEventHandle {
    /// Struct name as `ADDRESS::MODULE_NAME::STRUCT_NAME<TypeParam1?, TypeParam2?>`
    /// Example: `0x123::event_test::WithdrawEvent --cursor 0 --limit 1`
    #[clap(long = "event_handle_type")]
    event_handle_type: StructTag,
    /// start position
    #[clap(long)]
    cursor: Option<u64>,
    /// Max number of items returned per page
    #[clap(long)]
    limit: Option<u64>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<EventPageViewResult> for GetEventsByEventHandle {
    async fn execute(self) -> RoochResult<EventPageViewResult> {
        let client = self.context_options.build().await?.get_client().await?;
        let resp = client
            .rooch
            .get_events_by_event_handle(self.event_handle_type.into(), self.cursor, self.limit)
            .await
            .map_err(RoochError::from)?;
        Ok(resp)
    }
}
