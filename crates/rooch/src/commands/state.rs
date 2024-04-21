// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::access_path::AccessPath;
use rooch_rpc_api::jsonrpc_types::StateView;
use rooch_types::error::{RoochError, RoochResult};

/// Get states by accessPath
#[derive(Parser)]
pub struct StateCommand {
    //TODO access path should support named address?
    /// /object/$object_id1[,$object_id2]
    /// /resource/$account_address/$resource_type1[,$resource_type2]
    /// /module/$account_address/$module_name1[,$module_name2]
    /// /table/$object_id/$key1[,$key2]
    #[clap(long = "access-path", short = 'a')]
    pub access_path: AccessPath,

    /// RPC client options.
    #[clap(flatten)]
    context_options: WalletContextOptions,

    /// Render and return display fields.
    #[clap(long)]
    pub show_display: bool,
}

#[async_trait]
impl CommandAction<Vec<Option<StateView>>> for StateCommand {
    async fn execute(self) -> RoochResult<Vec<Option<StateView>>> {
        let client = self.context_options.build()?.get_client().await?;

        let resp = if self.show_display {
            client
                .rooch
                .get_decoded_states_with_display(self.access_path)
                .await
                .map_err(RoochError::from)?
        } else {
            client
                .rooch
                .get_decoded_states(self.access_path)
                .await
                .map_err(RoochError::from)?
        };
        Ok(resp)
    }
}
