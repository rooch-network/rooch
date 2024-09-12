// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use clap::Parser;
use log::info;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use moveos_types::move_types::FunctionId;
use moveos_types::transaction::MoveAction;
use rooch_rpc_client::wallet_context::WalletContext;
use serde_json::Value;
use tokio::time::Instant;
use crate::data_process::{execute_transaction, parse_and_convert, subscribe_websocket, State};

#[derive(Parser, Debug, Clone)]
pub struct BinanceConfig {
    #[arg(long, default_value = "wss://stream.binance.com:9443/ws/btcusdt@ticker")]
    pub binance_url: String,

    #[arg(long, env = "ROOCH_BINANCE_WALLET_DIR", default_value = "~/.rooch/rooch_config")]
    pub binance_wallet_dir: Option<PathBuf>,

    #[arg(long, env = "ROOCH_BINANCE_WALLET_PWD")]
    pub binance_wallet_pwd: Option<String>,

    #[arg(long, default_value = "10")]
    pub binance_submit_interval: u64,

    #[arg(long, env = "ROOCH_BINANCE_ORACLE_ID")]
    pub binance_oracle_id: String,

    #[arg(long, env = "ROOCH_BINANCE_ADMIN_ID")]
    pub binance_admin_id: String,
}

pub struct Binance {
    pub wallet_state: Arc<RwLock<State>>,
    binance_config: BinanceConfig
}

impl Binance {
    pub async fn new(
        config: BinanceConfig,
    ) -> Self {
        let wallet = WalletContext::new(config.binance_wallet_dir.clone()).unwrap();
        let wallet_pwd = config.binance_wallet_pwd.clone();
        Self {
            wallet_state: Arc::new(RwLock::new(State {
                wallet_pwd,
                context: wallet,
            })),
            binance_config: config,
        }
    }

    pub async fn subscribe(
        &self,
        package_id: &str,
    ) {
        let (tx, mut rx) = mpsc::channel(1);
        let url = self.binance_config.binance_url.clone();
        let handle = tokio::spawn(async move {
            subscribe_websocket(url, tx, None).await;
        });
        let function_id = FunctionId::new(
            ModuleId::new(AccountAddress::from_hex_literal(package_id).unwrap(), Identifier::new("trusted_oracle").unwrap()),
            Identifier::new("submit_data").unwrap(),
        );
        let address_mapping = self.wallet_state.read().await.context.address_mapping.clone();
        let oracle_obj = parse_and_convert(format!("object_id:{}", self.binance_config.binance_oracle_id).as_str(), &address_mapping);
        let ticker = parse_and_convert("string:BTCUSD", &address_mapping);
        let identifier = parse_and_convert("string:Binance", &address_mapping);
        let admin_obj = parse_and_convert(format!("object_id:{}", self.binance_config.binance_admin_id).as_str(), &address_mapping);
        let mut last_execution = Instant::now() - Duration::from_secs(10); // 初始化为10秒前
        while let Some(msg) = rx.recv().await {
            let wallet_state = self.wallet_state.write().await;

            let msg_value = serde_json::from_str::<Value>(&msg).unwrap();
            if msg_value["c"].as_str().is_none() || Instant::now().duration_since(last_execution) < Duration::from_secs(self.binance_config.binance_submit_interval) {
                continue;
            }
            last_execution = Instant::now();
            let price = format!("u256:{}", msg_value["c"].as_str().unwrap().parse::<f64>().unwrap() * 10f64.powi(8));
            let decimal = "8u8".to_string();
            let args = vec![
                oracle_obj.clone(),
                ticker.clone(),
                parse_and_convert(price.as_str(), &address_mapping),
                parse_and_convert(decimal.as_str(), &address_mapping),
                identifier.clone(),
                admin_obj.clone()
            ];
            let move_action = MoveAction::new_function_call(
                function_id.clone(),
                vec![],
                args,
            );
            let _ = execute_transaction(move_action, wallet_state).await;
            info!("Received Binance price: {}", msg_value["c"]);
        }
        handle.await.expect("The task failed");
    }
}