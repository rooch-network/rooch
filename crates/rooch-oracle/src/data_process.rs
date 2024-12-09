// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use futures_util::{SinkExt, Stream, StreamExt};
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::KeptVMStatusView;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::RoochAddress;
use rooch_types::framework::oracle::OracleModule;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

use crate::datasource::OracleDecimalData;

pub fn subscribe_websocket(
    url: String,
    subscribe_msg: Option<Value>,
) -> impl Stream<Item = Result<Value>> {
    let (tx, rx) = mpsc::channel(1);

    tokio::spawn(async move {
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

            if let Some(msg) = &subscribe_msg {
                if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                    warn!("Failed to send message: {}", e);
                    continue;
                }
            }

            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        debug!("Received message: {}", text);
                        if let Err(e) =
                            tx.try_send(serde_json::from_str(&text).map_err(|e| anyhow::anyhow!(e)))
                        {
                            match e {
                                TrySendError::Full(_) => {
                                    debug!("Channel is full, discarding message");
                                }
                                TrySendError::Closed(_) => {
                                    warn!("Receiver dropped, stopping WebSocket connection");
                                    return;
                                }
                            }
                        }
                    }
                    Ok(msg) => {
                        debug!("Received message: {}, skip", msg);
                    }
                    Err(e) => {
                        error!("Failed to read message: {}", e);
                        break;
                    }
                }
            }

            warn!("Connection lost or error occurred, restarting...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    ReceiverStream::new(rx)
}

pub fn subscribe_http(url: String, interval: u64) -> impl Stream<Item = Result<Value>> {
    let (tx, rx) = mpsc::channel(1);

    tokio::spawn(async move {
        loop {
            match reqwest::get(&url).await {
                Ok(response) => {
                    if let Err(e) = tx.try_send(
                        response
                            .json::<Value>()
                            .await
                            .map_err(|e| anyhow::anyhow!(e)),
                    ) {
                        match e {
                            mpsc::error::TrySendError::Closed(_) => {
                                warn!("Subscribe {} channel closed", url);
                                return;
                            }
                            mpsc::error::TrySendError::Full(_) => {
                                debug!("Subscribe {} channel is full, skipping message", url);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch from url {}, error: {}", url, e);
                }
            };

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

const RETRY_INTERVAL: u64 = 5;

pub fn subscribe_http_stream(url: String) -> impl Stream<Item = Result<Value>> {
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        loop {
            info!("Fetching from url: {}", url);
            let client = reqwest::Client::new();
            match client.get(&url).send().await {
                Ok(response) => {
                    debug!("response: {response:?}");
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();
                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(bytes) => {
                                let chunk = String::from_utf8_lossy(&bytes);
                                for part in chunk.split('\n') {
                                    if part.starts_with("data:") {
                                        if !buffer.is_empty() {
                                            debug!("text {}", buffer);
                                            if let Err(e) = tx.try_send(
                                                serde_json::from_str(&buffer)
                                                    .map_err(|e| anyhow::anyhow!(e)),
                                            ) {
                                                match e {
                                                    TrySendError::Full(_) => {
                                                        debug!(
                                                            "Channel is full, discarding message"
                                                        );
                                                    }
                                                    TrySendError::Closed(_) => {
                                                        warn!("Receiver dropped, stopping WebSocket connection");
                                                        return;
                                                    }
                                                }
                                            }
                                            buffer = String::new();
                                        }
                                        buffer.push_str(part.strip_prefix("data:").unwrap_or(part));
                                    } else {
                                        buffer.push_str(part);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to read message: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch from url {}, error: {}", url, e);
                }
            };

            tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL)).await;
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

pub async fn execute_submit_data_tx(
    wallet_context: &WalletContext,
    sender: RoochAddress,
    oracle_id: ObjectID,
    ticker: String,
    data: OracleDecimalData,
    identifier: String,
    admin_obj: ObjectID,
) -> Result<()> {
    let action = OracleModule::submit_decimal_data_action(
        oracle_id,
        ticker,
        data.value,
        data.decimal,
        identifier,
        data.timestamp,
        admin_obj,
    );
    let tx_data = wallet_context.build_tx_data(sender, action, None).await?;
    let result = wallet_context.sign_and_execute(sender, tx_data).await;
    match result {
        Ok(tx) => match tx.execution_info.status {
            KeptVMStatusView::Executed => {
                info!(
                    "Submit data value: {}, timestamp:{}, tx_hash: {}, gas_used:{}",
                    data.value,
                    data.timestamp,
                    tx.execution_info.tx_hash,
                    tx.execution_info.gas_used
                );
            }
            _ => {
                bail!(
                    "Execute submit function error {:?}",
                    tx.execution_info.status
                );
            }
        },
        Err(e) => {
            bail!("Execute submit function error {:?}", e);
        }
    };
    Ok(())
}
