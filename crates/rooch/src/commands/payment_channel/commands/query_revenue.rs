// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::{DynamicField, ObjectID};
use rooch_rpc_api::jsonrpc_types::StrView;
use rooch_types::address::{ParsedAddress, RoochAddress};
use rooch_types::error::RoochResult;
use rooch_types::framework::multi_coin_store::CoinStoreField;
use rooch_types::framework::payment_revenue::{PaymentRevenueHub, PaymentRevenueModule};
use serde::{Deserialize, Serialize};

/// Query payment revenue information
#[derive(Debug, Parser)]
pub struct QueryRevenueCommand {
    /// DID address of the revenue hub owner (the DID document address)
    #[clap(long, help = "DID address of the revenue hub owner")]
    pub owner: ParsedAddress,

    /// Filter by specific revenue source type (optional)
    #[clap(long, help = "Filter by specific revenue source type")]
    pub source_type: Option<String>,

    /// Coin type to query (optional, defaults to RGas)
    #[clap(
        long,
        help = "Coin type to query",
        default_value = "0x3::gas_coin::RGas",
        value_parser=ParsedStructType::parse
    )]
    pub coin_type: ParsedStructType,

    /// Page size for listing fields
    #[clap(long, default_value = "100", help = "Page size for listing fields")]
    pub page_size: u64,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

// Output structs
#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueQueryOutput {
    pub hub_id: ObjectID,
    pub owner: RoochAddress,
    pub revenue_balances: Vec<RevenueBalanceInfo>,
    pub total_amount: Option<StrView<U256>>, // Only set when querying specific source type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueBalanceInfo {
    pub coin_type: String,
    pub amount: StrView<U256>,
    pub source_type: String,
    pub source_breakdown: Vec<SourceBreakdown>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceBreakdown {
    pub source_id: Option<String>, // ObjectID as string if present
    pub amount: StrView<U256>,
}

#[async_trait]
impl CommandAction<RevenueQueryOutput> for QueryRevenueCommand {
    async fn execute(self) -> RoochResult<RevenueQueryOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let owner = context.resolve_rooch_address(self.owner)?;

        // 1. Calculate revenue hub ID
        let hub_id = PaymentRevenueModule::payment_revenue_hub_id(owner.into());

        // 2. Get PaymentRevenueHub object state
        let mut hub_object_views = client
            .rooch
            .get_object_states(vec![hub_id.clone()], None)
            .await?;

        if hub_object_views.is_empty() || hub_object_views.first().unwrap().is_none() {
            // No revenue hub exists yet - return empty result
            return Ok(RevenueQueryOutput {
                hub_id,
                owner,
                revenue_balances: vec![],
                total_amount: None,
            });
        }

        let hub_object_view = hub_object_views.pop().unwrap().unwrap();
        let payment_revenue_hub = bcs::from_bytes::<PaymentRevenueHub>(&hub_object_view.value.0)
            .map_err(|_| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Failed to deserialize PaymentRevenueHub".to_string(),
                )
            })?;

        // 3. Get revenue balances from multi_coin_store (similar to hub query)
        let multi_coin_store_id = payment_revenue_hub.multi_coin_store();
        let coin_type = self.coin_type.into_struct_tag(&context.address_mapping())?;

        // 4. List fields in multi_coin_store to get all coin types and balances
        let mut revenue_balances = Vec::new();
        let mut total_revenue_amount = U256::zero();

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
                // Parse both coin types to compare them properly (handle address format differences)
                let field_coin_type = field
                    .name
                    .parse::<move_core_types::language_storage::StructTag>()
                    .ok();
                let requested_coin_type = Some(coin_type.clone());

                // Only include the requested coin type or all if no specific type requested
                if field_coin_type == requested_coin_type {
                    let balance = field.value.balance();
                    if balance > U256::zero() {
                        revenue_balances.push(RevenueBalanceInfo {
                            coin_type: field.name.clone(),
                            amount: balance.into(),
                            source_type: "payment_channel".to_string(), // For now, only payment_channel generates revenue
                            source_breakdown: vec![SourceBreakdown {
                                source_id: None, // Could be expanded to show individual channel IDs
                                amount: balance.into(),
                            }],
                        });
                        total_revenue_amount = balance;
                    }
                }
            }

            if !field_states.has_next_page {
                break;
            }
            cursor = field_states.next_cursor;
        }

        // 5. If querying specific source type, return total amount
        if let Some(_source_type) = &self.source_type {
            let total_amount = if total_revenue_amount > U256::zero() {
                Some(total_revenue_amount.into())
            } else {
                Some(U256::zero().into())
            };

            return Ok(RevenueQueryOutput {
                hub_id,
                owner,
                revenue_balances: vec![],
                total_amount,
            });
        }

        Ok(RevenueQueryOutput {
            hub_id,
            owner,
            revenue_balances,
            total_amount: None,
        })
    }
}
