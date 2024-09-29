// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;
use move_core_types::u256::U256;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    pin::Pin,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Ticker {
    #[default]
    BTCUSD,
}

impl FromStr for Ticker {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "BTCUSD" => Ok(Ticker::BTCUSD),
            _ => Err(anyhow::anyhow!("Invalid ticker")),
        }
    }
}

impl Display for Ticker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ticker::BTCUSD => write!(f, "BTCUSD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleDecimalData {
    pub value: U256,
    pub decimal: u8,
}

#[async_trait]
pub trait DataSource {
    const IDENTIFIER: &'static str;
    fn subscribe(
        self,
        ticker: Ticker,
    ) -> Pin<Box<dyn Stream<Item = Result<OracleDecimalData>> + Send + 'static>>;

    fn identifier(&self) -> &'static str {
        Self::IDENTIFIER
    }
}

pub mod binance;
pub mod okx;
pub mod pyth;

#[derive(Debug, Clone)]
pub enum DataSourceType {
    OKX(okx::OKXSource),
    Binance(binance::BinanceSource),
    Pyth(pyth::PythSource),
}

impl DataSourceType {
    pub fn identifier(&self) -> &'static str {
        match self {
            DataSourceType::OKX(ds) => ds.identifier(),
            DataSourceType::Binance(ds) => ds.identifier(),
            DataSourceType::Pyth(ds) => ds.identifier(),
        }
    }

    pub fn subscribe(
        self,
        ticker: Ticker,
    ) -> Pin<Box<dyn Stream<Item = Result<OracleDecimalData>> + Send + 'static>> {
        match self {
            DataSourceType::OKX(ds) => ds.subscribe(ticker),
            DataSourceType::Binance(ds) => ds.subscribe(ticker),
            DataSourceType::Pyth(ds) => ds.subscribe(ticker),
        }
    }
}

impl FromStr for DataSourceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            okx::OKXSource::IDENTIFIER => Ok(DataSourceType::OKX(okx::OKXSource)),
            binance::BinanceSource::IDENTIFIER => {
                Ok(DataSourceType::Binance(binance::BinanceSource))
            }
            pyth::PythSource::IDENTIFIER => Ok(DataSourceType::Pyth(pyth::PythSource)),
            _ => Err(anyhow::anyhow!("Invalid DataSourceType")),
        }
    }
}

impl fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;
    use tracing::{info, warn};

    async fn test_datasource(ds: impl DataSource) {
        let identifier = ds.identifier();
        let mut stream = ds.subscribe(Ticker::BTCUSD);
        //consume 10 items
        let mut ok_count = 0;
        for _ in 0..10 {
            let data = stream.next().await;
            if let Some(result) = data {
                match result {
                    Ok(data) => {
                        info!("Received data: {:?}", data);
                        ok_count += 1;
                    }
                    Err(e) => {
                        warn!("Error: {}", e);
                    }
                }
            } else {
                info!("No data received");
            }
        }
        assert!(ok_count > 0, "DataSource {} all failed", identifier);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_okx_datasource() {
        let _trace = tracing_subscriber::fmt().try_init();
        test_datasource(okx::OKXSource).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_binance_datasource() {
        let _trace = tracing_subscriber::fmt().try_init();
        test_datasource(binance::BinanceSource).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_pyth_datasource() {
        let _trace = tracing_subscriber::fmt().try_init();
        test_datasource(pyth::PythSource).await;
    }
}
