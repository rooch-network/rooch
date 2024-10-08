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

const URL_TEMPLATE: &str =
    "https://hermes.pyth.network/v2/updates/price/stream?ids[]={ticker}&parsed=true";

#[derive(Debug, Clone)]
pub struct PythSource;

impl DataSource for PythSource {
    const IDENTIFIER: &'static str = "pyth";
    fn subscribe(
        self,
        ticker: Ticker,
    ) -> Pin<Box<dyn Stream<Item = Result<OracleDecimalData>> + Send + 'static>> {
        let url = URL_TEMPLATE.replace("{ticker}", ticker_mapping(ticker));

        let stream = data_process::subscribe_http_stream(url);
        let stream = stream.map(|result| result.and_then(parse_data));
        Box::pin(stream)
    }
}

//https://www.pyth.network/developers/price-feed-ids
fn ticker_mapping(ticker: Ticker) -> &'static str {
    match ticker {
        Ticker::BTCUSD => "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
    }
}

fn parse_data(response: Value) -> Result<OracleDecimalData> {
    let price = response["parsed"][0]["ema_price"]["price"]
        .as_str()
        .ok_or_else(|| anyhow!("price field not found in response: {}", response))?
        .parse::<U256>()?;
    let publish_time = response["parsed"][0]["ema_price"]["publish_time"]
        .as_u64()
        .ok_or_else(|| anyhow!("publish_time field not found in response: {}", response))?;
    Ok(OracleDecimalData {
        value: price,
        decimal: 8,
        timestamp: publish_time * 1000,
    })
}
