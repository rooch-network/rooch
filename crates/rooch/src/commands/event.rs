// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use move_command_line_common::types::ParsedStructType;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::{EventOptions, EventPageView, StrView, StructTagOrObjectIDView};
use rooch_types::error::{RoochError, RoochResult};
use std::str::FromStr;

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
    /// Event handle as either:
    /// 1. Struct name: `ADDRESS::MODULE_NAME::STRUCT_NAME<TypeParam1?, TypeParam2?>`
    ///    Example: `0x123::event_test::WithdrawEvent`
    /// 2. Event handle ID: `0x123...abc`
    #[clap(short = 't', long = "event-handle")]
    event_handle: String,
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
        let client = context.get_client().await?;

        // Parse the event handle as either StructTag or ObjectID
        // Need handle the adderss mapping, so we can't use StructTagOrObjectIDView::from_str directly
        let event_handle =
            // Try parsing as StructTag first
            match ParsedStructType::parse(&self.event_handle) {
                Ok(parsed_type) => {
                    let struct_tag = parsed_type.into_struct_tag(&address_mapping)?;
                    StructTagOrObjectIDView::StructTag(StrView(struct_tag))
                },
                Err(_) => {
                    // If not a valid StructTag, try parsing as ObjectID
                    let object_id = ObjectID::from_str(&self.event_handle)?;
                    StructTagOrObjectIDView::ObjectID(StrView(object_id))
                }
            };

        let resp = client
            .rooch
            .get_events_by_event_handle(
                event_handle,
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
