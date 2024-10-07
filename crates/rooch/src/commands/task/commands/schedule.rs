// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    checker::{Checker, ViewFunctionCheckerArgs},
    runner::{FunctionRunnerArgs, Runner},
};
use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

/// Run a schedule task
#[derive(Debug, Parser)]
pub struct ScheduleCommand {
    /// The interval in seconds to check the task
    #[clap(long = "checker-interval", default_value = "10")]
    pub checker_interval: u64,

    #[clap(flatten)]
    pub checker: ViewFunctionCheckerArgs,

    #[clap(flatten)]
    pub runner: FunctionRunnerArgs,

    /// Transaction options for the task runner
    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Stop the schedule task on checker error
    #[clap(long, default_value = "false")]
    stop_on_checker_error: bool,

    /// Stop the schedule task on runner error
    #[clap(long, default_value = "false")]
    stop_on_runner_error: bool,

    /// Stop the schedule task after executed times
    #[clap(long)]
    stop_after_executed_times: Option<u64>,
}

#[async_trait]
impl CommandAction<String> for ScheduleCommand {
    async fn execute(self) -> RoochResult<String> {
        let context = self.context_options.build_require_password()?;
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let check_interval = self.checker_interval;
        let checker = self.checker.build(&context).await?;
        let runner = self
            .runner
            .build(sender, self.tx_options.max_gas_amount, &context)
            .await?;
        let mut schedule_task =
            tokio::time::interval(tokio::time::Duration::from_secs(check_interval));
        let mut executed_times = 0u64;
        let stop_after_executed_times = self.stop_after_executed_times.unwrap_or(u64::MAX);
        loop {
            schedule_task.tick().await;
            let check_result = checker.check().await;
            match check_result {
                Ok(result) => {
                    if result {
                        match runner.run().await {
                            Ok(_) => {
                                executed_times += 1;
                                if executed_times >= stop_after_executed_times {
                                    return Ok(format!(
                                        "Task executed {} times, stop the schedule task",
                                        executed_times
                                    ));
                                }
                                continue;
                            }
                            Err(e) => {
                                if self.stop_on_runner_error {
                                    return Err(e.into());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if self.stop_on_checker_error {
                        return Err(e.into());
                    }
                }
            }
        }
    }
}
