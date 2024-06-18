// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use move_command_line_common::types::ParsedStructType;
use moveos_types::access_path::AccessPath;
use rooch_rpc_api::jsonrpc_types::{
    IndexerObjectStatePageView, ObjectStateFilterView, QueryOptions, RoochAddressView, StateView,
};
use rooch_types::address::ParsedAddress;
use rooch_types::{error::RoochResult, function_arg::ParsedObjectID};

#[derive(Parser)]
pub struct ObjectCommand {
    #[clap(subcommand)]
    cmd: OjbectSubCommand,
}

#[derive(Subcommand)]
pub enum OjbectSubCommand {
    Get(GetObjectCommand),
    Query(QueryObjectCommand),
}

#[async_trait]
impl CommandAction<String> for ObjectCommand {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            OjbectSubCommand::Get(c) => c.execute_serialized().await,
            OjbectSubCommand::Query(c) => c.execute_serialized().await,
        }
    }
}

/// Get object by object id
#[derive(Debug, Parser)]
pub struct GetObjectCommand {
    /// Object id.
    #[clap(long)]
    pub id: ParsedObjectID,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,

    /// Render and return display fields.
    #[clap(long, default_value = "false")]
    pub show_display: bool,
}

#[async_trait]
impl CommandAction<Option<StateView>> for GetObjectCommand {
    async fn execute(self) -> RoochResult<Option<StateView>> {
        let context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let id = self.id.into_object_id(&mapping)?;
        let client = context.get_client().await?;
        let resp = if self.show_display {
            client
                .rooch
                .get_decoded_states_with_display(AccessPath::object(id))
                .await?
                .pop()
                .flatten()
        } else {
            client
                .rooch
                .get_decoded_states(AccessPath::object(id))
                .await?
                .pop()
                .flatten()
        };
        Ok(resp)
    }
}

#[derive(Parser)]
pub struct QueryObjectCommand {
    #[clap(subcommand)]
    filter: ObjectStateFilterOptions,
}

#[derive(Parser)]
pub struct QueryObjectOptions {
    /// Max number of items returned per page
    #[clap(long)]
    limit: Option<usize>,

    /// descending order
    #[clap(short = 'd', long, default_value = "false")]
    descending_order: bool,

    /// Render and return display fields.
    #[clap(long, default_value = "false")]
    show_display: bool,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

/// Query object by filter
#[derive(Parser)]
pub enum ObjectStateFilterOptions {
    /// Query by object value type and owner.
    #[clap(name = "type-owner")]
    ObjectTypeWithOwner {
        #[clap(flatten)]
        options: QueryObjectOptions,

        /// Struct name as `ADDRESS::MODULE_NAME::STRUCT_NAME<TypeParam1?, TypeParam2?>`
        #[clap(short = 't', long, value_parser=ParsedStructType::parse)]
        object_type: ParsedStructType,

        /// The address of the object's owner.
        #[clap(short = 'o', long, value_parser=ParsedAddress::parse, default_value = "default")]
        owner: ParsedAddress,
    },

    /// Query by object value type.
    #[clap(name = "type")]
    ObjectType {
        #[clap(flatten)]
        options: QueryObjectOptions,

        /// Struct name as `ADDRESS::MODULE_NAME::STRUCT_NAME<TypeParam1?, TypeParam2?>`
        #[clap(short = 't', long, value_parser=ParsedStructType::parse)]
        object_type: ParsedStructType,
    },

    /// Query by owner.
    #[clap(name = "owner")]
    Owner {
        #[clap(flatten)]
        options: QueryObjectOptions,

        /// The address of the object's owner.
        #[clap(short = 'o', long, value_parser=ParsedAddress::parse, default_value = "default")]
        owner: ParsedAddress,
    },

    /// Query by object ids.
    #[clap(name = "object-ids")]
    ObjectId {
        #[clap(flatten)]
        options: QueryObjectOptions,

        /// Object ids. Separate multiple IDs with a space.
        #[clap(short = 'i', long, value_delimiter = ' ', num_args = 1..)]
        object_ids: Vec<ParsedObjectID>,
    },
}

#[async_trait]
impl CommandAction<IndexerObjectStatePageView> for QueryObjectCommand {
    async fn execute(self) -> RoochResult<IndexerObjectStatePageView> {
        match self.filter {
            ObjectStateFilterOptions::ObjectTypeWithOwner {
                options,
                object_type,
                owner,
            } => {
                let context = options.context_options.build()?;
                let address_mapping = context.address_mapping();
                let client = context.get_client().await?;

                let obj_type = object_type.into_struct_tag(&address_mapping)?;
                let owner_addr: RoochAddressView =
                    owner.into_rooch_address(&address_mapping)?.into();
                let filter = ObjectStateFilterView::ObjectTypeWithOwner {
                    object_type: obj_type.into(),
                    owner: owner_addr.into(),
                };

                let query_options = QueryOptions {
                    descending: options.descending_order,
                    decode: true,
                    show_display: options.show_display,
                };
                Ok(client
                    .rooch
                    .query_object_states(filter, None, options.limit, Some(query_options))
                    .await?)
            }
            ObjectStateFilterOptions::ObjectType {
                options,
                object_type,
            } => {
                let context = options.context_options.build()?;
                let address_mapping = context.address_mapping();
                let client = context.get_client().await?;

                let obj_type = object_type.into_struct_tag(&address_mapping)?;
                let filter = ObjectStateFilterView::ObjectType(obj_type.into());

                let query_options = QueryOptions {
                    descending: options.descending_order,
                    decode: true,
                    show_display: options.show_display,
                };
                Ok(client
                    .rooch
                    .query_object_states(filter, None, options.limit, Some(query_options))
                    .await?)
            }
            ObjectStateFilterOptions::Owner { options, owner, .. } => {
                let context = options.context_options.build()?;
                let address_mapping = context.address_mapping();
                let client = context.get_client().await?;

                let owner_addr: RoochAddressView =
                    owner.into_rooch_address(&address_mapping)?.into();
                let filter = ObjectStateFilterView::Owner(owner_addr.into());

                let query_options = QueryOptions {
                    descending: options.descending_order,
                    decode: true,
                    show_display: options.show_display,
                };
                Ok(client
                    .rooch
                    .query_object_states(filter, None, options.limit, Some(query_options))
                    .await?)
            }
            ObjectStateFilterOptions::ObjectId {
                options,
                object_ids,
                ..
            } => {
                let context = options.context_options.build()?;
                let address_mapping = context.address_mapping();
                let client = context.get_client().await?;

                let obj_ids = object_ids
                    .into_iter()
                    .map(|id| id.into_object_id(&address_mapping))
                    .collect::<Result<Vec<_>>>()?;
                let filter = ObjectStateFilterView::ObjectId(
                    obj_ids.into_iter().map(|id| id.into()).collect(),
                );

                let query_options = QueryOptions {
                    descending: options.descending_order,
                    decode: true,
                    show_display: options.show_display,
                };
                Ok(client
                    .rooch
                    .query_object_states(filter, None, options.limit, Some(query_options))
                    .await?)
            }
        }
    }
}
