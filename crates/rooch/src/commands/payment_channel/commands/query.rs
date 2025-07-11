// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use move_core_types::u256::U256;
use moveos_types::move_std::option::MoveOption;
use moveos_types::moveos_std::object::{DynamicField, ObjectID};
use rooch_rpc_api::jsonrpc_types::StrView;
use rooch_types::address::{ParsedAddress, RoochAddress};
use rooch_types::error::RoochResult;
use rooch_types::framework::multi_coin_store::CoinStoreField;
use rooch_types::framework::payment_channel::{
    CancellationInfo, PaymentChannelModule, PaymentHub, SubChannel,
};
use serde::{Deserialize, Serialize};

/// Query payment channel information
#[derive(Debug, Parser)]
pub struct QueryCommand {
    #[clap(subcommand)]
    pub query_type: QueryType,
}

#[derive(Debug, Parser)]
pub enum QueryType {
    /// Query payment hub information
    #[clap(name = "hub")]
    Hub(HubCommand),
    /// Query payment channel information
    #[clap(name = "channel")]
    Channel(ChannelCommand),
}

/// Query payment hub information
#[derive(Debug, Parser)]
pub struct HubCommand {
    /// Address of the hub owner
    #[clap(long, help = "Address of the hub owner")]
    pub owner: ParsedAddress,

    /// Page size for listing fields
    #[clap(long, default_value = "100", help = "Page size for listing fields")]
    pub page_size: u64,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

/// Query payment channel information
#[derive(Debug, Parser)]
pub struct ChannelCommand {
    /// Channel ID to query
    #[clap(long, help = "Channel ID to query")]
    pub channel_id: ObjectID,

    /// List all sub-channels
    #[clap(long, help = "List all sub-channels")]
    pub list_sub_channels: bool,

    /// Query specific sub-channel by VM ID fragment
    #[clap(long, help = "Query specific sub-channel by VM ID fragment")]
    pub vm_id_fragment: Option<String>,

    /// Page size for listing fields
    #[clap(long, default_value = "100", help = "Page size for listing fields")]
    pub page_size: u64,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

/// Non-generic PaymentChannel data structure for deserialization
#[derive(Debug, Serialize, Deserialize)]
struct PaymentChannelData {
    pub sender: AccountAddress,
    pub receiver: AccountAddress,
    pub payment_hub_id: ObjectID,
    pub sub_channels: ObjectID, // Table handle
    pub status: u8,
    pub cancellation_info: MoveOption<CancellationInfo>,
}

// Output structs
#[derive(Debug, Serialize, Deserialize)]
pub struct HubOutput {
    pub hub_id: ObjectID,
    pub owner: RoochAddress,
    pub balances: Vec<BalanceInfo>,
    pub active_channels: Vec<ActiveChannelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub coin_type: String,
    pub amount: String, // Use string to handle u256
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveChannelInfo {
    pub coin_type: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelOutput {
    pub channel_id: ObjectID,
    pub sender: RoochAddress,
    pub receiver: RoochAddress,
    pub payment_hub_id: ObjectID,
    pub status: String,
    pub cancellation_info: Option<CancellationInfoOutput>,
    pub sub_channels_count: u64,
    pub sub_channels: Option<Vec<SubChannelInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancellationInfoOutput {
    pub initiated_time: u64,
    pub pending_amount: String, // Use string to handle u256
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubChannelInfo {
    pub fragment: String,
    pub pk_multibase: String,
    pub method_type: String,
    pub last_claimed_amount: StrView<U256>,
    pub last_confirmed_nonce: u64,
}

impl From<DynamicField<String, SubChannel>> for SubChannelInfo {
    fn from(field: DynamicField<String, SubChannel>) -> Self {
        Self {
            fragment: field.name,
            pk_multibase: field.value.pk_multibase.to_string(),
            method_type: field.value.method_type.to_string(),
            last_claimed_amount: field.value.last_claimed_amount.into(),
            last_confirmed_nonce: field.value.last_confirmed_nonce,
        }
    }
}

#[async_trait]
impl CommandAction<serde_json::Value> for QueryCommand {
    async fn execute(self) -> RoochResult<serde_json::Value> {
        match self.query_type {
            QueryType::Hub(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
            QueryType::Channel(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
        }
    }
}

impl QueryCommand {
    pub async fn execute_serialized(self) -> RoochResult<String> {
        let result = self.execute().await?;
        Ok(serde_json::to_string(&result)?)
    }
}

#[async_trait]
impl CommandAction<HubOutput> for HubCommand {
    async fn execute(self) -> RoochResult<HubOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let owner = context.resolve_address(self.owner.into())?;
        // 1. Calculate hub_id directly in Rust
        let hub_id = PaymentChannelModule::payment_hub_id(owner);

        // 2. Get PaymentHub object state
        let mut hub_object_views = client
            .rooch
            .get_object_states(vec![hub_id.clone()], None)
            .await?;

        if hub_object_views.is_empty() || hub_object_views.first().unwrap().is_none() {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                format!("Payment hub for address {} not found", owner),
            ));
        }

        let hub_object_view = hub_object_views.pop().unwrap().unwrap();
        let payment_hub =
            bcs::from_bytes::<PaymentHub>(&hub_object_view.value.0).map_err(|_| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Failed to deserialize PaymentHub".to_string(),
                )
            })?;

        // 3. Get balances from multi_coin_store
        let mut balances = Vec::new();
        let multi_coin_store_id = payment_hub.multi_coin_store();

        // List fields in multi_coin_store to get all coin types and balances
        let mut cursor = None;
        loop {
            let field_states = client
                .rooch
                .list_field_states(
                    multi_coin_store_id.clone().into(),
                    cursor,
                    Some(self.page_size),
                    None,
                )
                .await?;

            for state in &field_states.data {
                let field =
                    bcs::from_bytes::<DynamicField<String, CoinStoreField>>(&state.state.value.0)
                        .map_err(|_| {
                        rooch_types::error::RoochError::CommandArgumentError(
                            "Failed to deserialize CoinStoreField".to_string(),
                        )
                    })?;

                balances.push(BalanceInfo {
                    coin_type: field.name,
                    amount: field.value.balance().to_string(),
                });
            }

            if !field_states.has_next_page {
                break;
            }
            cursor = field_states.next_cursor;
        }

        // 4. Get active_channels from the active_channels table
        let mut active_channels = Vec::new();
        let active_channels_table_id = payment_hub.active_channels();

        // List fields in active_channels table to get coin types and their counts
        let mut cursor = None;
        loop {
            let field_states = client
                .rooch
                .list_field_states(
                    active_channels_table_id.clone().into(),
                    cursor,
                    Some(self.page_size),
                    None,
                )
                .await?;

            for state in &field_states.data {
                // The key is coin_type (String), value is count (u64)
                let field = bcs::from_bytes::<DynamicField<String, u64>>(&state.state.value.0)
                    .map_err(|_| {
                        rooch_types::error::RoochError::CommandArgumentError(
                            "Failed to deserialize u64".to_string(),
                        )
                    })?;

                active_channels.push(ActiveChannelInfo {
                    coin_type: field.name,
                    count: field.value,
                });
            }

            if !field_states.has_next_page {
                break;
            }
            cursor = field_states.next_cursor;
        }

        Ok(HubOutput {
            hub_id,
            owner: owner.into(),
            balances,
            active_channels,
        })
    }
}

#[async_trait]
impl CommandAction<ChannelOutput> for ChannelCommand {
    async fn execute(self) -> RoochResult<ChannelOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        let mut channel_object_views = client
            .rooch
            .get_object_states(vec![self.channel_id.clone()], None)
            .await?;

        if channel_object_views.is_empty() || channel_object_views.first().unwrap().is_none() {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                format!("Payment channel {} not found", self.channel_id),
            ));
        }

        let channel_object_view = channel_object_views.pop().unwrap().unwrap();
        let payment_channel = bcs::from_bytes::<PaymentChannelData>(&channel_object_view.value.0)
            .map_err(|_| {
            rooch_types::error::RoochError::CommandArgumentError(
                "Failed to deserialize PaymentChannel".to_string(),
            )
        })?;

        // 3. Convert status to string
        let status = match payment_channel.status {
            0 => "Active",
            1 => "Cancelling",
            2 => "Closed",
            _ => "Unknown",
        };

        // 4. Convert cancellation_info if present
        let cancellation_info =
            payment_channel
                .cancellation_info
                .as_ref()
                .map(|info| CancellationInfoOutput {
                    initiated_time: info.initiated_time(),
                    pending_amount: info.pending_amount().to_string(),
                });

        // 5. Get sub-channels if requested
        let mut sub_channels_count = 0u64;
        let sub_channels = if self.list_sub_channels || self.vm_id_fragment.is_some() {
            let mut sub_channels_info = Vec::new();
            let sub_channels_table_id = payment_channel.sub_channels;

            // Query specific sub-channel by vm_id
            if let Some(vm_id) = &self.vm_id_fragment {
                // Query specific field by key
                let field_key = moveos_types::state::FieldKey::derive_from_string(vm_id);
                let field_states = client
                    .rooch
                    .get_field_states(sub_channels_table_id.into(), vec![field_key.into()], None)
                    .await?;

                for state in field_states.into_iter().flatten() {
                    let sub_channel_field =
                        bcs::from_bytes::<DynamicField<String, SubChannel>>(&state.value.0)
                            .map_err(|_| {
                                rooch_types::error::RoochError::CommandArgumentError(
                                    "Failed to deserialize SubChannel".to_string(),
                                )
                            })?;
                    sub_channels_info.push(sub_channel_field.into());
                    sub_channels_count += 1;
                }
            } else if self.list_sub_channels {
                // List all sub-channels with pagination
                let mut cursor = None;
                loop {
                    let field_states = client
                        .rooch
                        .list_field_states(
                            sub_channels_table_id.clone().into(),
                            cursor,
                            Some(self.page_size),
                            None,
                        )
                        .await?;

                    for state in &field_states.data {
                        let sub_channel_field =
                            bcs::from_bytes::<DynamicField<String, SubChannel>>(
                                &state.state.value.0,
                            )
                            .map_err(|_| {
                                rooch_types::error::RoochError::CommandArgumentError(
                                    "Failed to deserialize SubChannel".to_string(),
                                )
                            })?;
                        sub_channels_info.push(sub_channel_field.into());
                        sub_channels_count += 1;
                    }

                    if !field_states.has_next_page {
                        break;
                    }
                    cursor = field_states.next_cursor;
                }
            }

            Some(sub_channels_info)
        } else {
            let mut sub_channels_table_object = client
                .rooch
                .get_object_states(vec![payment_channel.sub_channels], None)
                .await?;
            if sub_channels_table_object.is_empty()
                || sub_channels_table_object.first().unwrap().is_none()
            {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!(
                        "Sub-channels table for channel {} not found",
                        self.channel_id
                    ),
                ));
            }
            let sub_channels_table_object_view = sub_channels_table_object.pop().unwrap().unwrap();
            let sub_channels_table_size = sub_channels_table_object_view.metadata.size;

            sub_channels_count = sub_channels_table_size.0;
            None
        };

        Ok(ChannelOutput {
            channel_id: self.channel_id,
            sender: payment_channel.sender.into(),
            receiver: payment_channel.receiver.into(),
            payment_hub_id: payment_channel.payment_hub_id,
            status: status.to_string(),
            cancellation_info,
            sub_channels_count,
            sub_channels,
        })
    }
}
