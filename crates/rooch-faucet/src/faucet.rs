// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{metrics::FaucetMetrics, FaucetError, FaucetRequest};
use anyhow::Result;
use clap::Parser;
use move_core_types::language_storage::StructTag;
use moveos_types::transaction::MoveAction;
use prometheus::Registry;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::{MultiChainAddress, RoochAddress};
use rooch_types::authentication_key::AuthenticationKey;
use rooch_types::framework::transfer::TransferModule;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock, RwLockWriteGuard,
};

use rooch_rpc_api::jsonrpc_types::KeptVMStatusView;
use rooch_types::error::RoochError;

pub const DEFAULT_AMOUNT: u64 = 1_000_000_000;

#[derive(Parser, Debug, Clone)]
pub struct FaucetConfig {
    /// The amount of funds to grant to each account on startup in Rooch.
    #[arg(
        long, default_value_t = DEFAULT_AMOUNT,
    )]
    pub faucet_grant_amount: u64,

    pub wallet_config_dir: Option<PathBuf>,

    #[clap(long, default_value_t = 10000)]
    pub max_request_queue_length: u64,

    pub(crate) session_key: Option<AuthenticationKey>,
}

impl Default for FaucetConfig {
    fn default() -> Self {
        Self {
            faucet_grant_amount: DEFAULT_AMOUNT,
            wallet_config_dir: None,
            session_key: None,
            max_request_queue_length: 1000,
        }
    }
}

struct State {
    config: FaucetConfig,
    wallet_pwd: Option<String>,
    context: WalletContext,
    // metrics: FaucetMetrics,
}

pub struct Faucet {
    state: Arc<RwLock<State>>,
    faucet_receiver: Arc<RwLock<Receiver<FaucetRequest>>>,
    faucet_error_sender: Sender<FaucetError>,
}

impl Faucet {
    pub async fn new(
        prometheus_registry: &Registry,
        config: FaucetConfig,
        faucet_receiver: Receiver<FaucetRequest>,
        faucet_error_sender: Sender<FaucetError>,
    ) -> Result<Self> {
        let wallet = WalletContext::new(config.wallet_config_dir.clone()).unwrap();
        let _metrics = FaucetMetrics::new(prometheus_registry);
        let wallet_pwd = match env::var("ROOCH_FAUCET_PWD") {
            Ok(val) => Some(val.parse::<String>().unwrap()),
            _ => None,
        };

        Ok(Self {
            state: Arc::new(RwLock::new(State {
                config,
                wallet_pwd,
                context: wallet,
            })),
            faucet_error_sender,
            faucet_receiver: Arc::new(RwLock::new(faucet_receiver)),
        })
    }

    pub async fn start(self) -> Result<()> {
        self.monitor_faucet_requests().await
    }

    async fn monitor_faucet_requests(&self) -> Result<()> {
        while let Some(request) = self.faucet_receiver.write().await.recv().await {
            match request {
                FaucetRequest::FixedBTCAddressRequest(req) => {
                    let mul_addr = MultiChainAddress::from(req.recipient);
                    self.transfer_gases_with_multi_addr(mul_addr)
                        .await
                        .expect("TODO: panic message");
                }
                FaucetRequest::FixedRoochAddressRequest(req) => {
                    self.transfer_gases(req.recipient)
                        .await
                        .expect("TODO: panic message");
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn execute_transaction<'a>(
        &self,
        action: MoveAction,
        state: RwLockWriteGuard<'a, State>,
    ) -> Result<()> {
        let sender: RoochAddress = state.context.client_config.active_address.unwrap();
        let pwd = state.wallet_pwd.clone();
        let result = if let Some(session_key) = state.config.session_key.clone() {
            let tx_data = state
                .context
                .build_tx_data(sender, action, None)
                .await
                .map_err(FaucetError::internal)?;
            let tx = state
                .context
                .keystore
                .sign_transaction_via_session_key(&sender, tx_data, &session_key, pwd)
                .map_err(|e| RoochError::SignMessageError(e.to_string()))
                .map_err(FaucetError::internal)?;
            state.context.execute(tx).await
        } else {
            state
                .context
                .sign_and_execute(sender, action, pwd, None)
                .await
        };

        match result {
            Ok(tx) => match tx.execution_info.status {
                KeptVMStatusView::Executed => {
                    tracing::info!(
                        "Transfer gases success tx_has: {}",
                        tx.execution_info.tx_hash
                    );
                }
                _ => {
                    let err = FaucetError::Transfer(format!("{:?}", tx.execution_info.status));
                    tracing::error!("Transfer gases failed {}", err);
                    if let Err(e) = self.faucet_error_sender.try_send(err) {
                        tracing::warn!("Failed to send error to faucet_error_sender: {:?}", e);
                    }
                }
            },
            Err(e) => {
                let err = FaucetError::transfer(e);
                tracing::error!("Transfer gases failed {}", err);
                if let Err(e) = self.faucet_error_sender.try_send(err) {
                    tracing::warn!("Failed to send error to faucet_error_sender: {:?}", e);
                }
            }
        };
        Ok(())
    }

    async fn transfer_gases_with_multi_addr(&self, recipient: MultiChainAddress) -> Result<()> {
        tracing::info!("transfer gases recipient: {}", recipient);

        let state = self.state.write().await;

        let move_action = TransferModule::create_transfer_coin_to_multichain_address_action(
            StructTag::from_str("0x3::gas_coin::GasCoin").unwrap(),
            recipient,
            state.config.faucet_grant_amount.into(),
        );

        self.execute_transaction(move_action, state).await
    }

    async fn transfer_gases(&self, recipient: RoochAddress) -> Result<()> {
        tracing::info!("transfer gases recipient: {}", recipient);

        let state = self.state.write().await;

        let move_action = TransferModule::create_transfer_coin_action(
            StructTag::from_str("0x3::gas_coin::GasCoin").unwrap(),
            recipient.into(),
            state.config.faucet_grant_amount.into(),
        );

        self.execute_transaction(move_action, state).await
    }
}
