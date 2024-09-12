// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use move_core_types::account_address::AccountAddress;
use moveos_types::transaction::MoveAction;
use rooch_rpc_api::jsonrpc_types::KeptVMStatusView;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::RoochAddress;
use rooch_types::function_arg::FunctionArg;
use serde_json::Value;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::{mpsc, RwLockWriteGuard};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn subscribe_websocket(
    url: String,
    tx: mpsc::Sender<String>,
    subscribe_msg: Option<Value>,
) {
    loop {
        let (ws_stream, _) = match connect_async(&url).await {
            Ok(stream) => stream,
            Err(e) => {
                warn!("Failed to connect: {} error:{}", url, e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("Connected to {}", url);

        let (mut write, mut read) = ws_stream.split();

        if subscribe_msg.is_some() {
            if let Err(e) = write
                .send(Message::Text(subscribe_msg.clone().unwrap().to_string()))
                .await
            {
                warn!("Failed to send message: {}", e);
                continue;
            }
        }

        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    if let Message::Text(text) = msg {
                        if let Err(e) = tx.send(text).await {
                            warn!("Failed to send message through channel: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error: {}", e);
                    break;
                }
            }
        }

        warn!("Connection lost or error occurred, restarting...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub async fn subscribe_http(url: String, tx: mpsc::Sender<Value>, interval: u64) {
    loop {
        match reqwest::get(&url).await {
            Ok(response) => {
                if let Ok(value) = response.json::<Value>().await {
                    if let Err(e) = tx.send(value).await {
                        warn!("Failed to send message through channel: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch price: {}", e);
            }
        };

        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

pub struct State {
    pub(crate) wallet_pwd: Option<String>,
    pub context: WalletContext,
}

pub async fn execute_transaction<'a>(
    action: MoveAction,
    state: RwLockWriteGuard<'a, State>,
) -> Result<()> {
    let sender: RoochAddress = state.context.client_config.active_address.unwrap();
    let pwd = state.wallet_pwd.clone();
    let result = state
        .context
        .sign_and_execute(sender, action, pwd, None)
        .await;
    match result {
        Ok(tx) => match tx.execution_info.status {
            KeptVMStatusView::Executed => {
                info!("Executed success tx_has: {}", tx.execution_info.tx_hash);
            }
            _ => {
                error!("Transfer gases failed {:?}", tx.execution_info.status);
            }
        },
        Err(e) => {
            error!("Transfer gases failed {}", e);
        }
    };
    Ok(())
}

pub fn parse_and_convert(arg: &str, address_mapping: &BTreeMap<String, AccountAddress>) -> Vec<u8> {
    let mapping = |input: &str| -> Option<AccountAddress> { address_mapping.get(input).cloned() };
    FunctionArg::from_str(arg)
        .unwrap()
        .into_bytes(&mapping)
        .unwrap()
}
