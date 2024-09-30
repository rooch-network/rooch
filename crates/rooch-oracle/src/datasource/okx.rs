// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{DataSource, OracleDecimalData, Ticker};
use crate::data_process;
use anyhow::{anyhow, Result};
use futures_util::Stream;
use move_core_types::u256::U256;
use serde_json::{json, Value};
use std::pin::Pin;
use tokio_stream::StreamExt;

const URL: &str = "wss://ws.okx.com:8443/ws/v5/public";

#[derive(Debug, Clone)]
pub struct OKXSource;

impl DataSource for OKXSource {
    const IDENTIFIER: &'static str = "okx";
    fn subscribe(
        self,
        ticker: Ticker,
    ) -> Pin<Box<dyn Stream<Item = Result<OracleDecimalData>> + Send + 'static>> {
        let inst_id = ticker_mapping(ticker);
        let subscribe_msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": "tickers",
                "instId": inst_id.to_string()
            }]
        });
        let url = URL.to_string();
        let stream = data_process::subscribe_websocket(url, Some(subscribe_msg));
        //skip the first message
        let stream = stream.skip(1).map(|result| result.and_then(parse_data));
        Box::pin(stream)
    }
}

fn ticker_mapping(ticker: Ticker) -> &'static str {
    match ticker {
        Ticker::BTCUSD => "BTC-USDT",
    }
}

fn parse_data(response: Value) -> Result<OracleDecimalData> {
    let last = response["data"][0]["last"]
        .as_str()
        .ok_or_else(|| anyhow!("last field not found in response: {}", response))?
        .parse::<f64>()?
        * 10f64.powi(8);
    Ok(OracleDecimalData {
        value: U256::from(last as u64),
        decimal: 8,
    })
}
