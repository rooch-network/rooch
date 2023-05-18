use async_trait::async_trait;
use clap::Parser;
use rooch_server::Service;
use rooch_types::cli::{CliError, CliResult, CommandAction};
use serde::{Deserialize, Serialize};
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

#[derive(Debug, clap::Subcommand)]
#[clap(name = "server")]
pub enum ServerCommand {
    Start(StartServer),
}

impl ServerCommand {
    pub async fn execute(self) -> CliResult<String> {
        match self {
            ServerCommand::Start(start) => start.execute_serialized().await,
        }
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct StartServer {}

#[async_trait]
impl CommandAction<()> for StartServer {
    async fn execute(self) -> CliResult<()> {
        let mut service = Service::new();
        service.start().await.map_err(CliError::from)?;

        let mut sig_int = signal(SignalKind::interrupt()).map_err(CliError::from)?;
        let mut sig_term = signal(SignalKind::terminate()).map_err(CliError::from)?;

        tokio::select! {
            _ = sig_int.recv() => info!("receive SIGINT"),
            _ = sig_term.recv() => info!("receive SIGTERM"),
            _ = ctrl_c() => info!("receive Ctrl C"),
        }

        service.stop().map_err(CliError::from)?;

        info!("Shutdown Sever");
        Ok(())
    }
}
