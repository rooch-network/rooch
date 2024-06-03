// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{
    path::PathBuf,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};

use crate::{DiscordConfig, FaucetError, FaucetRequest};

use rooch_rpc_api::jsonrpc_types::StructTagView;
use rooch_rpc_client::wallet_context::WalletContext;

#[derive(Clone, Debug)]
pub struct App {
    pub faucet_queue: Sender<FaucetRequest>,
    pub err_receiver: Arc<RwLock<Receiver<FaucetError>>>,
    pub wallet_config_dir: Option<PathBuf>,
    pub discord_config: DiscordConfig,
    pub faucet_funds: u64,
    pub is_loop_running: Arc<AtomicBool>,
}

impl App {
    pub fn new(
        faucet_queue: Sender<FaucetRequest>,
        wallet_config_dir: Option<PathBuf>,
        discord_config: DiscordConfig,
        err_receiver: Receiver<FaucetError>,
        faucet_funds: u64,
    ) -> Self {
        Self {
            faucet_queue,
            wallet_config_dir,
            discord_config,
            faucet_funds,
            is_loop_running: Arc::new(AtomicBool::new(false)),
            err_receiver: Arc::new(RwLock::new(err_receiver)),
        }
    }

    pub async fn request(&self, address: FaucetRequest) -> Result<(), FaucetError> {
        self.faucet_queue
            .send(address)
            .await
            .map_err(FaucetError::internal)?;
        Ok(())
    }

    pub async fn check_gas_balance(&self) -> Result<f64, FaucetError> {
        let context = WalletContext::new(self.wallet_config_dir.clone())
            .map_err(|e| FaucetError::Wallet(e.to_string()))?;
        let client = context.get_client().await.unwrap();
        let faucet_address = context.client_config.active_address.unwrap();

        let s = client
            .rooch
            .get_balance(
                faucet_address.into(),
                StructTagView::from_str("0x3::gas_coin::GasCoin").unwrap(),
            )
            .await
            .map_err(FaucetError::internal)?;

        let divisor: u64 = 10u64.pow(s.coin_info.decimals as u32);
        let result = s.balance.0.unchecked_as_u64() as f64 / divisor as f64;

        Ok(result)
    }
}
