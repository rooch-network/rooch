// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::FaucetMetrics;
use crate::FaucetError;
use anyhow::Result;
use clap::Parser;
use move_core_types::language_storage::StructTag;
use prometheus::Registry;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::{BitcoinAddress, MultiChainAddress, RoochAddress};
use rooch_types::authentication_key::AuthenticationKey;
use rooch_types::framework::transfer::TransferModule;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, RwLock};

use rooch_rpc_api::jsonrpc_types::KeptVMStatusView;
use rooch_types::error::{RoochError, RoochResult};

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
    faucet_receiver: Arc<RwLock<Receiver<BitcoinAddress>>>,
}

impl Faucet {
    pub async fn new(
        prometheus_registry: &Registry,
        config: FaucetConfig,
        faucet_receiver: Receiver<BitcoinAddress>,
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
                // metrics,
            })),
            faucet_receiver: Arc::new(RwLock::new(faucet_receiver)),
        })
    }

    pub async fn start(self) -> Result<(), FaucetError> {
        self.monitor_faucet_requests().await
    }

    async fn monitor_faucet_requests(&self) -> Result<(), FaucetError> {
        while let Some(address) = self.faucet_receiver.write().await.recv().await {
            if let Err(e) = self.transfer_gases(address).await {
                tracing::error!("Transfer gases failed {}", e)
            }
        }

        Ok(())
    }

    // TODO: check balance
    // async fn monitor_check_ba(&self) -> Result<()> {
    //     loop {
    //     }
    // }

    // ./rooch move run --function 0x3::transfer::transfer_coin_to_multichain_address --type-args 0x3::gas_coin::GasCoin --args 0u64 --args address:bcrt1pyltxam359x8fryn0pnr9xvd7g6prxvgs48wd859hqx2w04ld59hq7pnlk4 100000u256
    // TODO: add queue bitch run ?
    // TODO: retry ?
    // TODO: record faucet address
    async fn transfer_gases(&self, recipient: BitcoinAddress) -> Result<(), FaucetError> {
        tracing::info!("transfer gases recipient: {}", recipient);

        let state = self.state.write().await;

        let sender: RoochAddress = state.context.client_config.active_address.unwrap();

        let move_action = TransferModule::create_transfer_coin_to_multichain_address_action(
            StructTag::from_str("0x3::gas_coin::GasCoin").unwrap(),
            MultiChainAddress::from(recipient),
            state.config.faucet_grant_amount.into(),
        );

        let pwd = state.wallet_pwd.clone();
        let result = if let Some(session_key) = state.config.session_key.clone() {
            let tx_data = state
                .context
                .build_tx_data(sender, move_action, None)
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
                .sign_and_execute(sender, move_action, pwd, None)
                .await
        };

        match result {
            Ok(tx) => match tx.execution_info.status {
                KeptVMStatusView::Executed => Ok(()),
                _ => Err(FaucetError::Transfer(format!(
                    "{:?}",
                    tx.execution_info.status
                ))),
            },
            Err(e) => Err(FaucetError::transfer(e)),
        }
    }
}
