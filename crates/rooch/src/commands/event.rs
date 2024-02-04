// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use move_command_line_common::types::ParsedStructType;
use rooch_rpc_api::jsonrpc_types::{EventOptions, EventPageView};
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
    #[clap(short = 't',long = "event-handle-type", value_parser=ParsedStructType::parse)]
    event_handle_type: ParsedStructType,
    /// start position
    #[clap(long)]
    cursor: Option<u64>,
    /// Max number of items returned per page
    #[clap(long)]
    limit: Option<u64>,
    /// descending order
    #[clap(short = 'd', long)]
    descending_order: Option<bool>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<EventPageView> for GetEventsByEventHandle {
    async fn execute(self) -> RoochResult<EventPageView> {
        let context = self.context_options.build()?;
        let address_mapping = context.address_mapping();
        let event_handle_type = self.event_handle_type.into_struct_tag(&address_mapping)?;
        let client = context.get_client().await?;
        let resp = client
            .rooch
            .get_events_by_event_handle(
                event_handle_type.into(),
                self.cursor,
                self.limit,
                self.descending_order,
                Some(EventOptions::default().decode(true)),
            )
            .await
            .map_err(RoochError::from)?;
        Ok(resp)
    }
}
