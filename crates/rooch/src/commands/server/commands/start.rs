// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_config::RoochOpt;
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
    #[clap(flatten)]
    opt: RoochOpt,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for StartCommand {
    async fn execute(self) -> RoochResult<()> {
        let mut service = Service::new();
        service
            .start(&self.opt.clone())
            .await
            .map_err(RoochError::from)?;

        //Automatically switch env when use start server, if network is local or dev seed
        let mut context = self.context_options.build().await?;
        let active_env = context.config.get_active_env()?;
        let rooch_chain_id = self.opt.chain_id.unwrap_or_default();
        let chain_name = rooch_chain_id.chain_name().to_lowercase();
        // When chain_id is not equals to env alias
        let switch_env = if active_env.alias.to_owned() != chain_name {
            if RoochChainID::LOCAL == rooch_chain_id {
                Some(RoochChainID::LOCAL.chain_name().to_lowercase())
            } else if RoochChainID::DEV == rooch_chain_id {
                Some(RoochChainID::DEV.chain_name().to_lowercase())
            } else {
                println!(
                    "Warning! The active env is not equals to chain_id when server start, current chain_id is `{}`, while active env is `{}`",
                    chain_name, active_env.alias
                );
                None
            }
        } else {
            None
        };

        if let Some(switch_env_alias) = switch_env.clone() {
            if context
                .config
                .get_env(&Some(switch_env_alias.clone()))
                .is_none()
            {
                return Err(RoochError::SwitchEnvError(format!(
                    "Auto switch env failed when server start, the env config for `{}` does not exist",
                    switch_env_alias
                )));
            }
            context.config.active_env = switch_env;
            context.config.save()?;
            println!(
                "The active env was successfully switched to `{}`",
                switch_env_alias
            );
        }

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
