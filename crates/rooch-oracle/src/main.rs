// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_oracle::binance::{Binance, BinanceConfig};
use rooch_oracle::okx::{Okx, OkxConfig};
use rooch_oracle::pyth::{Pyth, PythConfig};

#[derive(Parser, Clone)]
#[clap(
    name = "Rooch Oracle",
    about = "Oracle backend for BTC tokens price on Rooch",
    rename_all = "kebab-case"
)]
pub struct Config {
    #[clap(flatten)]
    pub okx_config: OkxConfig,
    #[clap(flatten)]
    pub binance_config: BinanceConfig,
    #[clap(flatten)]
    pub pyth_config: PythConfig,
    #[clap(short, long, env = "ROOCH_ORACLE_PACKAGE")]
    pub package_id: String,
}

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let config = Config::parse();
    let Config {
        okx_config,
        binance_config,
        pyth_config,
        package_id,
    } = config;
    let okx_handle = tokio::spawn({
        let package_id = package_id.clone();
        async move {
            let okx = Okx::new(okx_config).await;
            okx.subscribe(package_id.as_str()).await;
        }
    });
    let binance_handle = tokio::spawn({
        let package_id = package_id.clone();
        async move {
            let binance = Binance::new(binance_config).await;
            binance.subscribe(package_id.as_str()).await;
        }
    });
    let pyth_handle = tokio::spawn({
        let package_id = package_id.clone();
        async move {
            let pyth = Pyth::new(pyth_config).await;
            pyth.subscribe(package_id.as_str()).await;
        }
    });

    okx_handle.await.expect("okx error");
    binance_handle.await.expect("binance error");
    pyth_handle.await.expect("binance error")
}
