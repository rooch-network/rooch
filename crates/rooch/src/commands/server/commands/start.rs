// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_rpc_server::Service;
use rooch_types::error::{RoochError, RoochResult};
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

/// Start service
#[derive(Debug, Parser)]
pub struct StartCommand {
    /// If true, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, parse(from_flag))]
    pub temp_db: bool,
}

#[async_trait]
impl CommandAction<()> for StartCommand {
    async fn execute(self) -> RoochResult<()> {
        let mut service = Service::new();
        service
            .start(self.temp_db)
            .await
            .map_err(RoochError::from)?;

        #[cfg(unix)]
        {
            let mut sig_int = signal(SignalKind::interrupt()).map_err(RoochError::from)?;
            let mut sig_term = signal(SignalKind::terminate()).map_err(RoochError::from)?;
            tokio::select! {
                _ = sig_int.recv() => info!("receive SIGINT"),
                _ = sig_term.recv() => info!("receive SIGTERM"),
                _ = ctrl_c() => info!("receive Ctrl C"),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::select! {
                _ = ctrl_c() => info!("receive Ctrl C"),
            }
        }

        service.stop().map_err(RoochError::from)?;

        info!("Shutdown Sever");
        Ok(())
    }
}
