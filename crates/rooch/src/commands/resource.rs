// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use moveos_types::access_path::AccessPath;
use rooch_rpc_api::jsonrpc_types::AnnotatedStateViewResult;
use rooch_types::error::RoochResult;

#[derive(Debug, Parser)]

/// Get account resource by tag
pub struct ResourceCommand {
    /// Account address where the resource stored.
    #[clap(long)]
    pub address: AccountAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam1?, TypeParam2?>`
    /// Example: `0x123::counter::Counter`, `0x123::counter::Box<0x123::counter::Counter>`
    #[clap(long = "resource")]
    pub resource: StructTag,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<AnnotatedStateViewResult>> for ResourceCommand {
    async fn execute(self) -> RoochResult<Option<AnnotatedStateViewResult>> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client
            .rooch
            .get_annotated_states(AccessPath::resource(self.address, self.resource))
            .await?
            .pop()
            .flatten();
        Ok(resp)
    }
}
