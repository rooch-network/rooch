// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::move_types::type_tag_match;
use rooch_rpc_api::jsonrpc_types::btc::ord::InscriptionFilterView;
use rooch_rpc_api::jsonrpc_types::btc::utxo::UTXOFilterView;
use rooch_rpc_api::jsonrpc_types::{ObjectStateFilterView, QueryOptions, RoochAddressView};
use rooch_types::address::ParsedAddress;
use rooch_types::indexer::state::{ObjectStateType, INSCRIPTION_TYPE_TAG, UTXO_TYPE_TAG, IndexerStateID};
use rooch_types::{error::RoochResult, function_arg::ParsedObjectID};

pub const QUERY_OBJECT_STATES_METHOD: &str = "rooch_queryObjectStates";
pub const QUERY_UTXOS_METHOD: &str = "btc_queryUTXOs";
pub const QUERY_INSCRIPTIONS_METHOD: &str = "btc_queryInscriptions";
#[derive(Parser)]
pub struct ObjectCommand {
    /// Object ids. Separate multiple IDs with a space.
    #[clap(short = 'i', long, value_delimiter = ' ', num_args = 1..)]
    object_ids: Option<Vec<ParsedObjectID>>,

    /// Struct name as `ADDRESS::MODULE_NAME::STRUCT_NAME<TypeParam1?, TypeParam2?>`
    #[clap(short = 't', long, value_parser=ParsedStructType::parse)]
    object_type: Option<ParsedStructType>,

    /// The address of the object's owner.
    #[clap(short = 'o', long, value_parser=ParsedAddress::parse)]
    owner: Option<ParsedAddress>,

    /// Provide the cursor in the format 'tx_order,state_index' (e.g., '12345,67890')
    #[clap(long, value_parser = clap::value_parser!(IndexerStateID))]
    cursor: Option<IndexerStateID>,

    /// Max number of items returned per page
    #[clap(long)]
    limit: Option<u64>,

    /// descending order
    #[clap(short = 'd', long, default_value = "false")]
    descending_order: bool,

    /// Render and return display fields.
    #[clap(long, default_value = "false")]
    pub show_display: bool,

    /// Is filter not object_type
    #[clap(long, default_value = "false")]
    pub filter_out: bool,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for ObjectCommand {
    async fn execute(self) -> RoochResult<String> {
        let context = self.context_options.build()?;
        let address_mapping = context.address_mapping();
        let client = context.get_client().await?;

        let mut filter: Option<ObjectStateFilterView> = None;
        if self.object_ids.is_some() {
            let object_ids = self.object_ids.clone().unwrap();

            let obj_ids = object_ids
                .into_iter()
                .map(|id| id.into_object_id(&address_mapping))
                .collect::<Result<Vec<_>>>()?;
            filter = Some(ObjectStateFilterView::ObjectId(obj_ids.into()));
        } else if self.owner.is_some() && self.object_type.is_some() {
            let owner = self.owner.clone().unwrap();
            let object_type = self.object_type.clone().unwrap();

            let obj_type = object_type.into_struct_tag(&address_mapping)?;
            let owner_addr: RoochAddressView = owner.into_rooch_address(&address_mapping)?.into();
            filter = Some(ObjectStateFilterView::ObjectTypeWithOwner {
                object_type: obj_type.into(),
                owner: owner_addr.into(),
            });
        } else if self.owner.is_some() {
            let owner = self.owner.clone().unwrap();

            let owner_addr: RoochAddressView = owner.into_rooch_address(&address_mapping)?.into();
            filter = Some(ObjectStateFilterView::Owner(owner_addr.into()));
        } else if self.object_type.is_some() {
            let object_type = self.object_type.clone().unwrap();

            let obj_type = object_type.into_struct_tag(&address_mapping)?;
            filter = Some(ObjectStateFilterView::ObjectType(obj_type.into()));
        }

        let object_state_type = if self.object_type.is_some() {
            let object_type = self.object_type.clone().unwrap();
            let obj_type = TypeTag::from(object_type.into_struct_tag(&address_mapping)?);
            if type_tag_match(&obj_type, &UTXO_TYPE_TAG) {
                ObjectStateType::UTXO
            } else if type_tag_match(&obj_type, &INSCRIPTION_TYPE_TAG) {
                ObjectStateType::Inscription
            } else {
                ObjectStateType::ObjectState
            }
        } else {
            ObjectStateType::ObjectState
        };

        let query_options = QueryOptions {
            descending: self.descending_order,
            decode: true,
            show_display: self.show_display,
            filter_out: self.filter_out,
        };

        if filter.is_none() {
            let context = self.context_options.build()?;
            let active_address: RoochAddressView =
                context.client_config.active_address.unwrap().into();
            filter = Some(ObjectStateFilterView::Owner(active_address.into()));
        }

        let output = match object_state_type {
            ObjectStateType::UTXO => {
                let utxo_fitler = match filter.unwrap() {
                    ObjectStateFilterView::ObjectTypeWithOwner {
                        object_type: _,
                        owner,
                    } => UTXOFilterView::Owner(owner),
                    ObjectStateFilterView::ObjectType(_object_type) => UTXOFilterView::All,
                    ObjectStateFilterView::Owner(owner) => UTXOFilterView::Owner(owner),
                    ObjectStateFilterView::ObjectId(object_id) => {
                        UTXOFilterView::ObjectId(object_id)
                    }
                };
                let result = client
                    .rooch
                    .query_utxos(
                        utxo_fitler,
                        self.cursor.clone(),
                        self.limit,
                        Some(query_options.descending),
                    )
                    .await?;
                serde_json::to_string_pretty(&result).unwrap()
            }
            ObjectStateType::Inscription => {
                let inscription_fitler = match filter.unwrap() {
                    ObjectStateFilterView::ObjectTypeWithOwner {
                        object_type: _,
                        owner,
                    } => InscriptionFilterView::Owner(owner),
                    ObjectStateFilterView::ObjectType(_object_type) => InscriptionFilterView::All,
                    ObjectStateFilterView::Owner(owner) => InscriptionFilterView::Owner(owner),
                    ObjectStateFilterView::ObjectId(object_id) => {
                        InscriptionFilterView::ObjectId(object_id)
                    }
                };
                let result = client
                    .rooch
                    .query_inscriptions(
                        inscription_fitler,
                        self.cursor.clone(),
                        self.limit,
                        Some(query_options),
                    )
                    .await?;
                serde_json::to_string_pretty(&result).unwrap()
            }
            ObjectStateType::ObjectState => {
                let result = client
                    .rooch
                    .query_object_states(
                        filter.unwrap(),
                        self.cursor.clone(),
                        self.limit,
                        Some(query_options),
                    )
                    .await?;
                serde_json::to_string_pretty(&result).unwrap()
            }
        };

        if output == "null" {
            return Ok("".to_string());
        }
        Ok(output)
    }
}
