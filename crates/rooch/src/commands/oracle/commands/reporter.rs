// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::moveos_std::object::ObjectID;
use rooch_oracle::aggregator_stream::AggregateStrategy;
use rooch_oracle::datasource::{DataSourceType, Ticker};
use rooch_oracle::reporter::Reporter;
use rooch_types::error::{RoochError, RoochResult};

/// Start a Oracle data reporter
#[derive(Debug, Parser)]
pub struct ReporterCommand {
    /// Report interval in seconds
    #[clap(long, default_value = "10")]
    pub report_interval: u64,

    /// The Oracle ObjectID
    #[clap(long, env = "ROOCH_ORACLE_ID")]
    pub oracle_id: ObjectID,

    /// The OracleAdminCap ObjectID
    #[clap(long, env = "ROOCH_ORACLE_ADMIN_ID")]
    pub oracle_admin_id: ObjectID,

    /// The DataSource Type
    #[clap(long)]
    pub data_source: DataSourceType,

    /// The ticker of the price data source
    #[clap(long, default_value_t)]
    pub ticker: Ticker,

    #[clap(long, default_value_t)]
    pub aggregate_strategy: AggregateStrategy,

    #[clap(long)]
    pub stop_on_error: bool,

    /// Stop after report N times
    #[clap(long)]
    pub stop_after_report_times: Option<u64>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for ReporterCommand {
    async fn execute(self) -> RoochResult<String> {
        let wallet_context = self.context_options.build_require_password()?;
        let sender = wallet_context
            .resolve_address(self.tx_options.sender)?
            .into();
        let reporter = Reporter::new(
            wallet_context,
            sender,
            self.oracle_id,
            self.oracle_admin_id,
            self.report_interval,
            self.ticker,
            self.data_source,
            self.aggregate_strategy,
            self.stop_on_error,
            self.stop_after_report_times,
        );
        reporter.run().await.map_err(RoochError::from)
    }
}
