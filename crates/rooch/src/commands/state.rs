// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use moveos_types::access_path::AccessPath;
use rooch_server::jsonrpc_types::AnnotatedStateView;
use rooch_types::error::{RoochError, RoochResult};

/// Get States by AccessPath
#[derive(clap::Parser)]
pub struct StateCommand {
    #[clap(long = "access-path", short = 'a')]
    pub access_path: AccessPath,

    /// RPC client options.
    #[clap(flatten)]
    context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Vec<Option<AnnotatedStateView>>> for StateCommand {
    async fn execute(self) -> RoochResult<Vec<Option<AnnotatedStateView>>> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client
            .get_annotated_states(self.access_path)
            .await
            .map_err(RoochError::from)?;
        Ok(resp)
    }
}
