// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::access_path::AccessPath;
use rooch_rpc_api::jsonrpc_types::StateView;
use rooch_types::{error::RoochResult, function_arg::ParsedObjectID};

/// Get object by object id
#[derive(Debug, Parser)]
pub struct ObjectCommand {
    /// Object id.
    #[clap(long)]
    pub id: ParsedObjectID,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<StateView>> for ObjectCommand {
    async fn execute(self) -> RoochResult<Option<StateView>> {
        let context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let id = self.id.into_object_id(&mapping)?;
        let client = context.get_client().await?;
        let resp = client
            .rooch
            .get_decoded_states(AccessPath::object(id))
            .await?
            .pop()
            .flatten();

        Ok(resp)
    }
}
