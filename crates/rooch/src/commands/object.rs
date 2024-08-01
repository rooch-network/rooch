// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use rooch_rpc_api::jsonrpc_types::{
    IndexerObjectStatePageView, ObjectStateFilterView, QueryOptions, RoochAddressView,
};
use rooch_types::address::ParsedAddress;
use rooch_types::{error::RoochResult, function_arg::ParsedObjectID};

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
impl CommandAction<IndexerObjectStatePageView> for ObjectCommand {
    async fn execute(self) -> RoochResult<IndexerObjectStatePageView> {
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
                filter_out: self.filter_out,
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

        let query_options = QueryOptions {
            descending: self.descending_order,
            decode: true,
            show_display: self.show_display,
        };

        if filter.is_none() {
            let context = self.context_options.build()?;
            let active_address: RoochAddressView =
                context.client_config.active_address.unwrap().into();
            filter = Some(ObjectStateFilterView::Owner(active_address.into()));
        }

        Ok(client
            .rooch
            .query_object_states(filter.unwrap(), None, self.limit, Some(query_options))
            .await?)
    }
}
