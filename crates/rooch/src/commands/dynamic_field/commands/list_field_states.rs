// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::{StateOptions, StatePageView};
use rooch_types::error::RoochResult;
use rooch_types::function_arg::ParsedObjectID;

/// List the dynamic field states of an object with an Object ID.
#[derive(Debug, Parser)]
pub struct ListFieldStatesCommand {
    #[clap(short = 'i', long, required = true)]
    object_id: ParsedObjectID,

    #[clap(long)]
    pub cursor: Option<String>,

    #[clap(long)]
    pub limit: Option<u64>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<StatePageView> for ListFieldStatesCommand {
    async fn execute(self) -> RoochResult<StatePageView> {
        let context = self.context_options.build()?;
        let address_mapping = context.address_mapping();
        let client = context.get_client().await?;

        let object_id = self.object_id.into_object_id(&address_mapping)?;
        let options = Some(StateOptions::new().decode(true));

        let resp = client
            .rooch
            .list_field_states(object_id.into(), self.cursor, self.limit, options)
            .await?;

        Ok(resp)
    }
}
