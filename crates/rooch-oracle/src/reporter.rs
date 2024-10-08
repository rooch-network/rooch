// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    aggregator_stream::{AggregateStrategy, AggregatorStream},
    data_process,
    datasource::{DataSourceType, Ticker},
};
use anyhow::Result;
use futures::stream::StreamExt;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::RoochAddress;
use tracing::error;

pub struct Reporter {
    pub wallet_context: WalletContext,
    pub sender: RoochAddress,
    pub oracle_id: ObjectID,
    pub oracle_admin_id: ObjectID,
    pub report_interval: u64,
    pub ticker: Ticker,
    pub aggregate_strategy: AggregateStrategy,
    pub datasource: DataSourceType,
    pub stop_on_error: bool,
    pub stop_after_report_times: Option<u64>,
}

impl Reporter {
    pub fn new(
        wallet_context: WalletContext,
        sender: RoochAddress,
        oracle_id: ObjectID,
        oracle_admin_id: ObjectID,
        report_interval: u64,
        ticker: Ticker,
        datasource: DataSourceType,
        aggregate_strategy: AggregateStrategy,
        stop_on_error: bool,
        stop_after_report_times: Option<u64>,
    ) -> Self {
        Self {
            wallet_context,
            sender,
            oracle_id,
            oracle_admin_id,
            report_interval,
            ticker,
            datasource,
            aggregate_strategy,
            stop_on_error,
            stop_after_report_times,
        }
    }

    pub async fn run(self) -> Result<String> {
        let Reporter {
            sender,
            oracle_id,
            oracle_admin_id,
            wallet_context,
            report_interval,
            ticker,
            datasource,
            aggregate_strategy,
            stop_on_error,
            stop_after_report_times,
        } = self;
        let identifier = datasource.identifier().to_owned();
        let stream = datasource.subscribe(ticker);
        let mut aggregator = AggregatorStream::new(stream, aggregate_strategy);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(report_interval));
        let mut report_times = 0u64;
        loop {
            let _ = interval.tick().await;
            if let Some(data) = aggregator.next().await {
                let result = data_process::execute_submit_data_tx(
                    &wallet_context,
                    sender,
                    oracle_id.clone(),
                    ticker.to_string(),
                    data,
                    identifier.clone(),
                    oracle_admin_id.clone(),
                )
                .await;
                if let Err(e) = result {
                    if stop_on_error {
                        return Err(e);
                    } else {
                        error!("Failed to submit data: {}", e);
                    }
                }
                report_times += 1;
                if let Some(stop_after_report_times) = stop_after_report_times {
                    if report_times >= stop_after_report_times {
                        break;
                    }
                }
            }
        }
        Ok(format!("Reported {} times", report_times))
    }
}
