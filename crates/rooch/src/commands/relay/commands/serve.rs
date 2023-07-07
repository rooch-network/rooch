// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_ws_relay::Service;
use rooch_types::error::{RoochError, RoochResult};
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;
use std::sync::mpsc as syncmpsc;
use std::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use rooch_ws_relay::server::config;
use console_subscriber::ConsoleLayer;

#[derive(Debug, Parser)]
pub struct ServeCommand {
    #[clap(long = "database")]
    /// Use a directory as the location of the database
    db: Option<String>,

    #[clap(long)]
    /// Use a file name as the location of the config file
    config: Option<String>,
}

#[async_trait]
impl CommandAction<()> for ServeCommand {
    async fn execute(self) -> RoochResult<()> {
        // get config file name from args
        let config_file_arg = self.config;

        let mut _log_guard: Option<WorkerGuard> = None;

        // configure settings from the config file (defaults to config.toml)
        // replace default settings with those read from the config file
        let mut settings = config::Settings::new(&config_file_arg);

        // setup tracing
        if settings.diagnostics.tracing {
            // enable tracing with tokio-console
            ConsoleLayer::builder().with_default_env().init();
        } else {
            // standard logging
            if let Some(path) = &settings.logging.folder_path {
                // write logs to a folder
                let prefix = match &settings.logging.file_prefix {
                    Some(p) => p.as_str(),
                    None => "relay",
                };
                let file_appender = tracing_appender::rolling::daily(path, prefix);
                let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
                let filter = EnvFilter::from_default_env();
                // assign to a variable that is not dropped till the program ends
                _log_guard = Some(guard);

                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_writer(non_blocking)
                    .try_init()
                    .unwrap();
            } else {
                // write to stdout
                tracing_subscriber::fmt::try_init().unwrap();
            }
        }
        info!("Serving Rooch ws relay");

        // get database directory from args
        let db_dir_arg = self.db;

        // update with database location from args, if provided
        if let Some(db_dir) = db_dir_arg {
            settings.database.data_directory = db_dir;
        }
        // we should have a 'control plane' channel to monitor and bump
        // the server.  this will let us do stuff like clear the database,
        // shutdown, etc.; for now all this does is initiate shutdown if
        // `()` is sent.  This will change in the future, this is just a
        // stopgap to shutdown the relay when it is used as a library.
        let (_, ctrl_rx): (MpscSender<()>, MpscReceiver<()>) = syncmpsc::channel();
        let mut service = Service::new();
        service.start(settings, ctrl_rx).map_err(RoochError::from)?;

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

        info!("Shutdown Relay");
        Ok(())
    }
}
