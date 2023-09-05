// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_config::RoochOpt;
use rooch_config::R_OPT_NET_HELP;
use rooch_rpc_server::Service;
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::{RoochError, RoochResult};
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

/// Start service
#[derive(Debug, Parser)]
pub struct StartCommand {
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    /// The port on which the server should listen defaults to `50051`
    #[clap(long, short = 'p')]
    pub port: Option<u16>,
}

#[async_trait]
impl CommandAction<()> for StartCommand {
    async fn execute(self) -> RoochResult<()> {
        let mut service = Service::new();
        let rooch_opt = RoochOpt {
            base_data_dir: None,
            chain_id: self.chain_id,
            store: None,
            port: self.port,
        };
        service.start(&rooch_opt).await.map_err(RoochError::from)?;

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
