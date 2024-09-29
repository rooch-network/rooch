// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{faucet_proxy::FaucetProxy, DiscordConfig, FaucetError, FaucetRequest};
use move_core_types::u256::U256;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::{mpsc::Receiver, RwLock};

#[derive(Clone, Debug)]
pub struct App {
    pub faucet_proxy: FaucetProxy,
    pub err_receiver: Arc<RwLock<Receiver<FaucetError>>>,
    pub discord_config: DiscordConfig,
    pub is_loop_running: Arc<AtomicBool>,
}

impl App {
    pub fn new(
        faucet_proxy: FaucetProxy,
        err_receiver: Receiver<FaucetError>,
        discord_config: DiscordConfig,
    ) -> Self {
        Self {
            faucet_proxy,
            err_receiver: Arc::new(RwLock::new(err_receiver)),
            discord_config,
            is_loop_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn request(&self, request: FaucetRequest) -> Result<U256, FaucetError> {
        let amount = self
            .faucet_proxy
            .claim(request.claimer)
            .await
            .map_err(FaucetError::internal)?;
        Ok(amount)
    }

    pub async fn check_gas_balance(&self) -> Result<U256, FaucetError> {
        let balance = self
            .faucet_proxy
            .balance()
            .await
            .map_err(FaucetError::internal)?;
        Ok(balance)
    }
}
