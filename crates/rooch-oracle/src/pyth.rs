// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::data_process::{execute_transaction, parse_and_convert, subscribe_http, State};
use clap::Parser;
use log::info;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use moveos_types::move_types::FunctionId;
use moveos_types::transaction::MoveAction;
use rooch_rpc_client::wallet_context::WalletContext;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Parser, Debug, Clone)]
pub struct PythConfig {
    #[arg(
        long,
        default_value = "https://hermes.pyth.network/v2/updates/price/latest?ids%5B%5D=0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43&ids%5B%5D=0xc96458d393fe9deb7a7d63a0ac41e2898a67a7750dbd166673279e06c868df0a"
    )]
    pub pyth_url: String,

    #[arg(
        long,
        env = "ROOCH_PYTH_WALLET_DIR",
        default_value = "~/.rooch/rooch_config"
    )]
    pub pyth_wallet_dir: Option<PathBuf>,

    #[arg(long, env = "ROOCH_PYTH_WALLET_PWD")]
    pub pyth_wallet_pwd: Option<String>,

    #[arg(long, default_value = "10")]
    pub pyth_submit_interval: u64,

    #[arg(long, env = "ROOCH_PYTH_ORACLE_ID")]
    pub pyth_oracle_id: String,

    #[arg(long, env = "ROOCH_PYTH_ADMIN_ID")]
    pub pyth_admin_id: String,
}

pub struct Pyth {
    pub wallet_state: Arc<RwLock<State>>,
    pyth_config: PythConfig,
}

impl Pyth {
    pub async fn new(config: PythConfig) -> Self {
        let wallet = WalletContext::new(config.pyth_wallet_dir.clone()).unwrap();
        let wallet_pwd = config.pyth_wallet_pwd.clone();
        Self {
            wallet_state: Arc::new(RwLock::new(State {
                wallet_pwd,
                context: wallet,
            })),
            pyth_config: config,
        }
    }
    pub async fn subscribe(&self, package_id: &str) {
        let (tx, mut rx) = mpsc::channel(1);
        let url = self.pyth_config.pyth_url.clone();
        let interval = self.pyth_config.pyth_submit_interval;
        let handle = tokio::spawn(async move {
            subscribe_http(url, tx, interval).await;
        });
        let function_id = FunctionId::new(
            ModuleId::new(
                AccountAddress::from_hex_literal(package_id).unwrap(),
                Identifier::new("trusted_oracle").unwrap(),
            ),
            Identifier::new("submit_data").unwrap(),
        );
        let address_mapping = self
            .wallet_state
            .read()
            .await
            .context
            .address_mapping
            .clone();
        let oracle_obj = parse_and_convert(
            format!("object_id:{}", self.pyth_config.pyth_oracle_id).as_str(),
            &address_mapping,
        );
        let ticker = parse_and_convert("string:BTCUSD", &address_mapping);
        let identifier = parse_and_convert("string:PYTH", &address_mapping);
        let admin_obj = parse_and_convert(
            format!("object_id:{}", self.pyth_config.pyth_admin_id).as_str(),
            &address_mapping,
        );
        while let Some(msg) = rx.recv().await {
            let wallet_state = self.wallet_state.write().await;
            let price = format!(
                "u256:{}",
                msg["parsed"][0]["ema_price"]["price"].as_str().unwrap()
            );
            let decimal = "8u8".to_string();
            let args = vec![
                oracle_obj.clone(),
                ticker.clone(),
                parse_and_convert(price.as_str(), &address_mapping),
                parse_and_convert(decimal.as_str(), &address_mapping),
                identifier.clone(),
                admin_obj.clone(),
            ];
            let move_action = MoveAction::new_function_call(function_id.clone(), vec![], args);
            let _ = execute_transaction(move_action, wallet_state).await;
            info!("Received Pyth price: {}", price);
        }
        handle.await.expect("The task failed");
    }
}
