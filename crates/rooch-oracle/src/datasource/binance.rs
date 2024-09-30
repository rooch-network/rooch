// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{DataSource, OracleDecimalData, Ticker};
use crate::data_process;
use anyhow::{anyhow, Result};
use futures_util::Stream;
use move_core_types::u256::U256;
use serde_json::Value;
use std::pin::Pin;
use tokio_stream::StreamExt;

const URL_TEMPLATE: &str = "wss://stream.binance.com:9443/ws/{ticker}@ticker";

#[derive(Debug, Clone)]
pub struct BinanceSource;

impl DataSource for BinanceSource {
    const IDENTIFIER: &'static str = "binance";
    fn subscribe(
        self,
        ticker: Ticker,
    ) -> Pin<Box<dyn Stream<Item = Result<OracleDecimalData>> + Send + 'static>> {
        let bticker = ticker_mapping(ticker);
        let url = URL_TEMPLATE.replace("{ticker}", bticker);
        let stream = data_process::subscribe_websocket(url, None);
        let stream = stream.map(|result| result.and_then(parse_data));
        Box::pin(stream)
    }
}

fn ticker_mapping(ticker: Ticker) -> &'static str {
    match ticker {
        Ticker::BTCUSD => "btcusdt",
    }
}

fn parse_data(response: Value) -> Result<OracleDecimalData> {
    let c = response["c"]
        .as_str()
        .ok_or_else(|| anyhow!("c field not found in response: {}", response))?
        .parse::<f64>()?
        * 10f64.powi(8);
    Ok(OracleDecimalData {
        value: U256::from(c as u64),
        decimal: 8,
    })
}
