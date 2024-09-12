// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::data_process::{execute_transaction, parse_and_convert, subscribe_websocket, State};
use clap::Parser;
use log::info;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use moveos_types::move_types::FunctionId;
use moveos_types::transaction::MoveAction;
use rooch_rpc_client::wallet_context::WalletContext;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;

#[derive(Parser, Debug, Clone)]
pub struct OkxConfig {
    #[arg(long, default_value = "wss://ws.okx.com:8443/ws/v5/public")]
    pub okx_url: String,

    #[arg(
        long,
        env = "ROOCH_OKX_WALLET_DIR",
        default_value = "~/.rooch/rooch_config"
    )]
    pub okx_wallet_dir: Option<PathBuf>,

    #[arg(long, env = "ROOCH_OKX_WALLET_PWD")]
    pub okx_wallet_pwd: Option<String>,

    #[arg(long, default_value = "10")]
    pub okx_submit_interval: u64,

    #[arg(long, env = "ROOCH_OKX_ORACLE_ID")]
    pub okx_oracle_id: String,

    #[arg(long, env = "ROOCH_OKX_ADMIN_ID")]
    pub okx_admin_id: String,
}

pub struct Okx {
    pub wallet_state: Arc<RwLock<State>>,
    okx_config: OkxConfig,
}

impl Okx {
    pub async fn new(config: OkxConfig) -> Self {
        let wallet = WalletContext::new(config.okx_wallet_dir.clone()).unwrap();
        let wallet_pwd = config.okx_wallet_pwd.clone();
        Self {
            wallet_state: Arc::new(RwLock::new(State {
                wallet_pwd,
                context: wallet,
            })),
            okx_config: config,
        }
    }

    pub async fn subscribe(&self, package_id: &str) {
        let subscribe_msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": "tickers",
                "instId": "BTC-USDT"
            }]
        });
        let (tx, mut rx) = mpsc::channel(1);
        let url = self.okx_config.okx_url.clone();
        let handle = tokio::spawn(async move {
            subscribe_websocket(url, tx, Some(subscribe_msg)).await;
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
            format!("object_id:{}", self.okx_config.okx_oracle_id).as_str(),
            &address_mapping,
        );
        let ticker = parse_and_convert("string:BTCUSD", &address_mapping);
        let identifier = parse_and_convert("string:OKX", &address_mapping);
        let admin_obj = parse_and_convert(
            format!("object_id:{}", self.okx_config.okx_admin_id).as_str(),
            &address_mapping,
        );
        let mut last_execution = Instant::now() - Duration::from_secs(10);
        while let Some(msg) = rx.recv().await {
            let wallet_state = self.wallet_state.write().await;
            let msg_value = serde_json::from_str::<Value>(&msg).unwrap();
            if msg_value["data"][0]["last"].as_str().is_none()
                || Instant::now().duration_since(last_execution)
                    < Duration::from_secs(self.okx_config.okx_submit_interval)
            {
                continue;
            }
            last_execution = Instant::now();
            let price = format!(
                "u256:{}",
                msg_value["data"][0]["last"]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    * 10f64.powi(8)
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
            info!(
                "Received Okex price: {:?}",
                msg_value["data"][0]["last"].as_str().unwrap()
            );
        }
        handle.await.expect("The task failed");
    }
}
