// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use moveos_types::access_path::AccessPath;
use rooch_client::Client;
use rooch_server::jsonrpc_types::AnnotatedStateView;
use rooch_types::cli::{CliError, CliResult, CommandAction};

/// Get States by AccessPath
#[derive(clap::Parser)]
pub struct StateCommand {
    #[clap(long = "access-path", short = 'a')]
    pub access_path: AccessPath,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

#[async_trait]
impl CommandAction<Vec<Option<AnnotatedStateView>>> for StateCommand {
    async fn execute(self) -> CliResult<Vec<Option<AnnotatedStateView>>> {
        let resp = self
            .client
            .get_annotated_states(self.access_path)
            .await
            .map_err(CliError::from)?;
        Ok(resp)
    }
}
