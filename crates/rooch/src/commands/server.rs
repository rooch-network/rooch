use clap::Parser;
use rooch_server::Service;
use rooch_types::cli::{CliError, CliResult};
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
    pub async fn execute(self) -> CliResult<()> {
        match self {
            ServerCommand::Start(start) => start.execute().await,
        }
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct StartServer {}

impl StartServer {
    pub async fn execute(self) -> CliResult<()> {
        let mut service = Service::new();
        service.start().await.map_err(CliError::from)?;

        let mut sig_int = signal(SignalKind::interrupt()).unwrap();
        let mut sig_term = signal(SignalKind::terminate()).unwrap();
        println!("Server started");
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
